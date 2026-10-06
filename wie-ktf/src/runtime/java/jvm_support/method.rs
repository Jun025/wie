use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::{
    fmt::{self, Debug, Formatter},
    mem::{offset_of, size_of},
    ops::{Deref, DerefMut},
};
use jvm::{ClassInstance, JavaError, JavaType, JavaValue, Jvm, Method, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::MethodAccessFlags;
use wipi_types::ktf::java::{
    JavaExceptionHandler as RawJavaExceptionHandler, JavaMethodDefinition as RawJavaMethod,
    JavaMethodExceptionTableEntry as RawJavaMethodExceptionTableEntry,
};

use alloc::sync::Arc;
use wie_core_arm::{
    Allocator, ArmCore, EmulatedFunction, EmulatedFunctionParam, RUN_FUNCTION_LR, RegisteredFunction, RegisteredFunctionHolder, ResultWriter, SvcId,
};
use wie_jvm_support::native::{NativeJavaValueCodec, decode_method_arguments, encode_method_arguments, method_argument_word_count};
use wie_util::{ByteWrite, Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic};

use crate::{
    emulator::IMAGE_BASE,
    runtime::{SVC_CATEGORY_JAVA, java::JavaSvcFunctions, java::jvm_support::JavaClassDefinition},
};

use super::{KtfJvmSupport, class_instance::JavaClassInstance, name::JavaFullName, value::JavaValueCodec};

pub struct JavaMethod {
    pub ptr_raw: u32,
    core: ArmCore,
}

impl JavaMethod {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore) -> Self {
        Self { ptr_raw, core: core.clone() }
    }

    pub fn new<C, Context>(
        core: &mut ArmCore,
        jvm: &Jvm,
        ptr_class: u32,
        proto: JavaMethodProto<C>,
        context: Context,
        java_functions: JavaSvcFunctions,
    ) -> Result<Self>
    where
        C: ?Sized + 'static + Send,
        Context: Deref<Target = C> + DerefMut + Clone + 'static + Sync + Send,
    {
        let full_name = JavaFullName::new(0, &proto.name, &proto.descriptor);
        let full_name_bytes = full_name.as_bytes();

        let ptr_name = Allocator::alloc(core, full_name_bytes.len() as u32)?;
        core.write_bytes(ptr_name, &full_name_bytes)?;

        let ptr_raw = Allocator::alloc(core, size_of::<RawJavaMethod>() as u32)?;

        let access_flags = proto.access_flags;
        let (fn_body, fn_body_native) = Self::register_java_method(core, jvm, ptr_raw, proto, context, java_functions)?;

        write_generic(
            core,
            ptr_raw,
            RawJavaMethod {
                fn_body,
                ptr_class,
                fn_body_native_or_exception_table: fn_body_native,
                ptr_name,
                exception_table_count: 0,
                unk3: 0,
                index_in_vtable: 0, // to be filled later
                access_flags: access_flags.bits(),
                unk6: 0,
            },
        )?;

        tracing::trace!("Wrote method {} at {ptr_raw:#x}", full_name.name());

        Ok(Self::from_raw(ptr_raw, core))
    }

    pub fn write_vtable_index(&mut self, new_index: u16) -> Result<()> {
        let mut raw: RawJavaMethod = read_generic(&self.core, self.ptr_raw)?;

        raw.index_in_vtable = new_index;

        write_generic(&mut self.core, self.ptr_raw, raw)?;

        Ok(())
    }

    pub fn ptr_class(&self) -> u32 {
        let raw: RawJavaMethod = read_generic(&self.core, self.ptr_raw).unwrap();

        raw.ptr_class
    }

    pub fn name(&self) -> Result<JavaFullName> {
        let raw: RawJavaMethod = read_generic(&self.core, self.ptr_raw)?;

        JavaFullName::from_ptr(&self.core, raw.ptr_name)
    }

    pub async fn run(&self, args: Box<[JavaValue]>) -> Result<JavaValue> {
        let raw: RawJavaMethod = read_generic(&self.core, self.ptr_raw)?;
        let return_type = JavaType::parse(&self.descriptor()).as_method().1.clone();

        let mut core = self.core.clone();

        let codec = JavaValueCodec::new(&self.core);
        let raw_args = encode_method_arguments(&codec, &args);

        struct JavaMethodRunResult {
            result: u32,
            result_high: u32,
        }

        impl wie_core_arm::RunFunctionResult<JavaMethodRunResult> for JavaMethodRunResult {
            fn get(core: &ArmCore) -> Self {
                let result = core.read_param(0).unwrap();
                let result_high = core.read_param(1).unwrap();

                Self { result, result_high }
            }
        }

        let access_flags = MethodAccessFlags::from_bits_truncate(raw.access_flags);

        // Re-enters `run_function` if a Java catch handler matches the current ARM frame —
        // mirrors the trampoline path in `interface.rs::map_jump_result`, but for the
        // outermost frame whose caller is the Rust JVM rather than another ARM trampoline.
        async fn run_with_unwind(core: &mut ArmCore, mut pc: u32, mut args: Vec<u32>) -> Result<JavaMethodRunResult> {
            let caller_context = core.save_context();

            loop {
                match core.run_function::<JavaMethodRunResult>(pc, &args).await {
                    Ok(r) => {
                        core.restore_context(&caller_context);
                        return Ok(r);
                    }
                    Err(WieError::JavaExceptionUnwind {
                        context_base,
                        target,
                        next_pc,
                    }) => {
                        tracing::debug!("Resuming via exception restore: pc={next_pc:#x}, context_base={context_base:#x}, target={target:#x}");
                        pc = next_pc;
                        args = vec![context_base, target];
                    }
                    Err(e) => {
                        let error = match e {
                            error @ WieError::JavaException(_) => error,
                            error => {
                                let context = core.dump_reg_stack(IMAGE_BASE);
                                match error {
                                    WieError::Unimplemented(message) => WieError::Unimplemented(format!("{message}{context}")),
                                    WieError::FatalError(message) => WieError::FatalError(format!("{message}{context}")),
                                    error => WieError::FatalError(format!("{error}{context}")),
                                }
                            }
                        };
                        core.restore_context(&caller_context);
                        return Err(error);
                    }
                }
            }
        }

        let relocated = crate::runtime::relocated::enter(&mut core)?;
        let result: Result<JavaMethodRunResult> = async {
            Ok(if access_flags.contains(MethodAccessFlags::NATIVE) {
                let arg_container = Allocator::alloc(&mut core, (raw_args.len() as u32) * 4)?;
                for (i, arg) in raw_args.iter().enumerate() {
                    write_generic(&mut core, arg_container + (i * 4) as u32, *arg)?;
                }

                tracing::trace!("Calling native method: {:#x}", raw.fn_body_native_or_exception_table);
                let result = run_with_unwind(&mut core, raw.fn_body_native_or_exception_table, vec![0, arg_container]).await;

                Allocator::free(&mut core, arg_container, (raw_args.len() as u32) * 4)?;

                result?
            } else {
                let mut params = vec![0];
                params.extend(raw_args);

                tracing::trace!("Calling method: {:#x}", raw.fn_body);
                run_with_unwind(&mut core, raw.fn_body, params).await?
            })
        }
        .await;
        if let Some(entry) = relocated {
            crate::runtime::relocated::leave(&mut core, entry)?;
        }
        let result = result?;

        if matches!(return_type, JavaType::Double | JavaType::Long) {
            Ok(codec.decode_wide(result.result, result.result_high, &return_type))
        } else {
            Ok(codec.decode_word(result.result, &return_type))
        }
    }

    fn exception_table(&self) -> Result<Vec<RawJavaMethodExceptionTableEntry>> {
        let raw: RawJavaMethod = read_generic(&self.core, self.ptr_raw)?;

        let mut result = Vec::with_capacity(raw.exception_table_count as _);

        if raw.exception_table_count == 0 {
            return Ok(result);
        }

        let mut cursor = raw.fn_body_native_or_exception_table;
        for _ in 0..raw.exception_table_count {
            let address = read_generic(&self.core, cursor)?;
            cursor += 4;

            result.push(read_generic(&self.core, address)?);
        }

        Ok(result)
    }

    pub(super) fn exception_class_matches(
        core: &ArmCore,
        jvm: &Jvm,
        exception: &dyn ClassInstance,
        relocated_names: Option<u32>,
        ptr_class: u32,
    ) -> Result<bool> {
        if ptr_class == 0 {
            return Ok(true);
        }
        // A relocated image names the class instead: `(index << 1) | 1` into its name table.
        if let Some(names) = relocated_names
            && ptr_class & 1 == 1
        {
            let ptr_name: u32 = read_generic(core, names + (ptr_class >> 1) * 4)?;
            let name = String::from_utf8_lossy(&read_null_terminated_string_bytes(core, ptr_name)?).into_owned();
            return Ok(jvm.is_instance(exception, &name));
        }

        if let Some(instance) = exception.as_any().downcast_ref::<JavaClassInstance>() {
            for class in instance.class()?.read_class_hierarchy()? {
                if class.ptr_raw == ptr_class || class.ptr_vtable()? == ptr_class {
                    return Ok(true);
                }
            }

            return Ok(false);
        }

        let class = JavaClassDefinition::from_raw(ptr_class, core);
        Ok(jvm.is_instance(exception, &class.name()?))
    }

    pub async fn handle_exception(core: &mut ArmCore, jvm: &Jvm, exception: Box<dyn ClassInstance>) -> Result<JavaMethodResult> {
        tracing::warn!("Java exception thrown: {exception:?}");

        // The handler records form a chain through `ptr_old_handler`, one per frame inside a try. A catch
        // that belongs to a CALLER frame is found by walking it, as the handset runtime did. Stopping at
        // the innermost record let the exception escape to the host while the dead frames' records stayed
        // registered, and the next throw read them: «Invalid memory access» in paint on two KTF titles and
        // «jump native address is null» on a third (docs/report/0445).
        let mut ptr_handler = KtfJvmSupport::current_java_exception_handler(core)?;
        // The relocated runtime's record keeps the try pc at +0x10 and has the catch read `e` from +0x38
        // (its try helper fills +0x0c with 0 and stores r4–r7, r8, sb, -, sl from +0x18 — `runtime::relocated`).
        let relocated = KtfJvmSupport::relocated_abi(core)?;
        let (pc_offset, exception_offset) = if relocated.is_some() {
            (0x10, 0x38)
        } else {
            (
                offset_of!(RawJavaExceptionHandler, current_pc) as u32,
                offset_of!(RawJavaExceptionHandler, unk3) as u32,
            )
        };
        // ponytail: a step cap instead of cycle detection — a chain this long is already corrupt.
        for _ in 0..4096 {
            if ptr_handler == 0 {
                break;
            }
            // A relocated image's record of a frame outside this host entry is the outer entry's to catch:
            // resuming it here would run that frame inside this entry's call.
            if relocated.is_some() && !crate::runtime::relocated::entry_owns_record(core, ptr_handler)? {
                break;
            }
            let exception_handler: RawJavaExceptionHandler = read_generic(core, ptr_handler)?;
            let current_pc: u32 = read_generic(core, ptr_handler + pc_offset)?;

            let method = JavaMethod::from_raw(exception_handler.ptr_method, core);
            let exception_table = method.exception_table()?;

            for entry in exception_table {
                if entry.from_pc <= current_pc
                    && current_pc < entry.to_pc
                    && Self::exception_class_matches(core, jvm, &*exception, relocated.map(|x| x.names), entry.ptr_class)?
                {
                    let restore_context: u32 = read_generic(core, exception_handler.ptr_functions + 4)?;
                    let contexts_base = ptr_handler + 24;

                    // The catch block reads its `e` from this slot (`wipi_types` calls it `unk3`), and no
                    // client.bin code writes it — the handset's throw did. Left 0, a handler that touches
                    // `e` throws NPE into its own still-registered try range: 43,276 catches in one paint
                    // on e9fac881e602 KTF (docs/report/0343).
                    write_generic(core, ptr_handler + exception_offset, KtfJvmSupport::class_instance_raw(&exception))?;
                    // The handler now runs at `target`, so the record says so. Compiled code writes this slot
                    // only where a range covers the handler body (a catch block covered by its `finally`); a
                    // `finally` that rethrows writes nothing, and with the try-range pc still here its own
                    // rethrow re-entered it forever — 86,010 throws in 3 s on dbd078113b97 KTF (docs/report/0448).
                    write_generic(core, ptr_handler + pc_offset, entry.target)?;
                    // The frames above the catching one are gone, and so are their records.
                    KtfJvmSupport::set_current_java_exception_handler(core, ptr_handler)?;

                    tracing::debug!(
                        "Java exception handler found: {:#x}, method: {:#x}",
                        entry.target,
                        exception_handler.ptr_method
                    );

                    // The record is this entry's (checked above), so the relocated runtime resumes it in
                    // place, as `map_exception_unwind` does for a jump: the handler's frame is below.
                    if relocated.is_some() {
                        return Ok(JavaMethodResult::new(vec![contexts_base, entry.target], Some(restore_context)));
                    }
                    return Err(WieError::JavaExceptionUnwind {
                        context_base: contexts_base,
                        target: entry.target,
                        next_pc: restore_context,
                    });
                }
            }
            ptr_handler = exception_handler.ptr_old_handler;
        }

        Err(WieError::JavaException(KtfJvmSupport::class_instance_raw(&exception)))
    }

    fn register_java_method<C, Context>(
        core: &mut ArmCore,
        jvm: &Jvm,
        ptr_method: u32,
        proto: JavaMethodProto<C>,
        context: Context,
        java_functions: JavaSvcFunctions,
    ) -> Result<(u32, u32)>
    where
        C: ?Sized + 'static + Send,
        Context: Deref<Target = C> + DerefMut + Clone + 'static + Sync + Send,
    {
        let java_type = JavaType::parse(&proto.descriptor);
        let (parameter_types, return_type) = java_type.as_method();

        let mut parameter_types = parameter_types.to_vec();
        if !proto.access_flags.contains(MethodAccessFlags::STATIC) {
            // TODO proper flag handling
            parameter_types.insert(0, JavaType::Class("".into())); // TODO name
        }

        let proxy = JavaMethodProxy {
            ptr_method,
            jvm: jvm.clone(),
            proto,
            context,
            parameter_types,
            return_type: return_type.clone(),
        };

        let proxy = Arc::new(Box::new(RegisteredFunctionHolder::new(proxy, &())) as Box<dyn RegisteredFunction>);
        java_functions.lock().insert(ptr_method, proxy.clone());

        // Entry-field addresses identify the ABI while sharing the method implementation.
        let fn_body = core.make_svc_stub(SVC_CATEGORY_JAVA, ptr_method)?;
        // AOT code selects the ABI using the original handset implementation,
        // which need not have the same native flag as our host prototype.
        // Host methods have no guest exception table, so both entries can coexist.
        let ptr_native_entry = ptr_method + offset_of!(RawJavaMethod, fn_body_native_or_exception_table) as u32;
        java_functions.lock().insert(ptr_native_entry, proxy);
        let fn_native = core.make_svc_stub(SVC_CATEGORY_JAVA, ptr_native_entry)?;

        Ok((fn_body, fn_native))
    }
}

#[async_trait::async_trait]
impl Method for JavaMethod {
    fn name(&self) -> String {
        let name = self.name().unwrap();

        name.name().into()
    }

    fn descriptor(&self) -> String {
        let name = self.name().unwrap();

        name.descriptor().into()
    }

    async fn run(&self, jvm: &Jvm, args: Box<[JavaValue]>) -> JvmResult<JavaValue> {
        let jvm_clone = jvm.clone();
        // A JVM→guest call is its own try scope: the guest's handler chain starts empty here and the
        // caller's is put back on the way out. A throw no guest frame catches leaves the call as a host
        // error, and the frames it skipped never unregister their records — without this the chain kept
        // pointing into their dead stack, and the next walk read whatever was written there since.
        let mut core = self.core.clone();
        let scope = KtfJvmSupport::enter_exception_scope(&mut core);
        let result = self.run(args).await;
        KtfJvmSupport::leave_exception_scope(&mut core, scope);
        match result {
            Ok(x) => Ok(x),
            Err(WieError::JavaException(x)) => Err(JavaError::JavaException(Box::new(JavaClassInstance::from_raw(x, &self.core)))),
            Err(WieError::JavaExceptionUnwind { .. }) => {
                Err(KtfJvmSupport::wie_error(&jvm_clone, "Java exception unwind crossed into JVM caller").await)
            }
            Err(x) => Err(KtfJvmSupport::wie_error(&jvm_clone, &x.to_string()).await),
        }
    }

    fn access_flags(&self) -> MethodAccessFlags {
        let raw: RawJavaMethod = read_generic(&self.core, self.ptr_raw).unwrap();

        MethodAccessFlags::from_bits_truncate(raw.access_flags)
    }
}

impl Debug for JavaMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("JavaMethod").field("ptr_raw", &self.ptr_raw).finish()
    }
}

struct JavaMethodProxy<C, Context>
where
    C: ?Sized + Send,
    Context: Deref<Target = C> + DerefMut + Clone,
{
    ptr_method: u32,
    jvm: Jvm,
    proto: JavaMethodProto<C>,
    context: Context,
    parameter_types: Vec<JavaType>,
    return_type: JavaType,
}

#[async_trait::async_trait]
impl<C, Context> EmulatedFunction<(), JavaMethodResult, ()> for JavaMethodProxy<C, Context>
where
    C: ?Sized + Send,
    Context: Deref<Target = C> + DerefMut + Clone + 'static + Sync + Send,
{
    async fn call(&self, core: &mut ArmCore, _: &mut ()) -> Result<JavaMethodResult> {
        let param_count = method_argument_word_count(&self.parameter_types);

        let native_entry = self.ptr_method + offset_of!(RawJavaMethod, fn_body_native_or_exception_table) as u32;
        let raw_args = if SvcId::get(core, 0).0 == native_entry {
            let param_base = u32::get(core, 1);
            (0..param_count)
                .map(|x| read_generic(core, param_base + (x as u32) * 4))
                .collect::<wie_util::Result<Vec<u32>>>()?
        } else {
            (0..param_count).map(|x| u32::get(core, x + 1)).collect::<Vec<_>>()
        };

        let codec = JavaValueCodec::new(core);
        let args = decode_method_arguments(&codec, &self.parameter_types, &raw_args);

        let mut context = self.context.clone();
        let (_, lr) = core.read_pc_lr()?;

        // A frame of its own, so what the method allocates is rooted only while it runs. Without it
        // everything landed in the frame of the host call the guest code was running under — for a
        // game loop, one that never returns, so nothing it made was ever collected (8d8c24b7c198:
        // the guest heap ran out after ~7.5 minutes). What the guest keeps is rooted by the guest
        // root scan instead (`wie_jvm_support::guest_roots`).
        wie_jvm_support::guest_roots::stress_collect(&self.jvm, core.id());
        self.jvm.push_native_frame();
        let result = self.proto.body.call(&self.jvm, &mut context, args.into_boxed_slice()).await;
        self.jvm.pop_frame();
        if let Err(JavaError::JavaException(x)) = result {
            // if we executed this from rust code, we should propagate this down
            if lr == RUN_FUNCTION_LR {
                let java_exception = KtfJvmSupport::class_instance_raw(&x);
                return Err(WieError::JavaException(java_exception));
            }
            return JavaMethod::handle_exception(core, &self.jvm, x).await;
        }

        let result = if matches!(self.return_type, JavaType::Double | JavaType::Long) {
            let (result, result_high) = codec.encode_wide(&result.unwrap());
            vec![result, result_high]
        } else {
            vec![codec.encode_word(&result.unwrap())]
        };

        Ok(JavaMethodResult { result, next_pc: None })
    }
}

pub struct JavaMethodResult {
    result: Vec<u32>,
    next_pc: Option<u32>,
}

impl JavaMethodResult {
    pub fn new(result: Vec<u32>, next_pc: Option<u32>) -> Self {
        Self { result, next_pc }
    }
}

impl ResultWriter<JavaMethodResult> for JavaMethodResult {
    fn write(self, core: &mut ArmCore, next_pc: u32) -> Result<()> {
        core.write_return_value(&self.result)?;

        if let Some(x) = self.next_pc {
            core.set_next_pc(x)?;
        } else {
            core.set_next_pc(next_pc)?;
        }

        Ok(())
    }
}
