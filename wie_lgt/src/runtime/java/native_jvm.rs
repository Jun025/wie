//! LGT native-backed JVM (PoC): the AOT-compiled app's classes are registered as
//! JVM classes whose instances are guest(ARM)-memory-backed and whose methods
//! dispatch to ARM code; and the platform classes the app imports are exposed to
//! the native code through `java_load_classes`' fixed-offset function-pointer
//! tables (native -> platform trampolines).
//!
//! Object bridge:
//!  - App instance: a guest object block; `this+0x08` -> zeroed field array (the
//!    layout the AOT code expects: `r1=[this,#8]; str rX,[r1, idx<<2]`).
//!  - `instances` maps `guest_ptr -> ClassInstance` so a guest pointer flowing
//!    through native code (a `this`, an arg, a return) round-trips to its JVM
//!    object. Platform objects returned to native get a small proxy block.
//!
//! Dispatch directions:
//!  - JVM -> native: `LgtMethod::run` marshals args into `r0..r3`,
//!    `run_function(code_ptr)`, marshals the return.
//!  - native -> platform: `java_load_classes` writes a trampoline pointer into
//!    each requested method slot (`static/virtual_method_offsets[idx*4]`); calling
//!    it re-enters the JVM and invokes the matching `wie_wipi_java`/`wie_midp`
//!    method by name+descriptor.
//!
//! See `docs/lgt_native_classes.md` (descriptor layout) and STEP2_REPORT.md.

use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    format,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use core::{
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
};

use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use jvm::{ClassDefinition, ClassInstance, Field, JavaError, JavaType, JavaValue, Jvm, Method, Result as JvmResult};
use spin::Mutex;

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore, SvcId};
use wie_jvm_support::JvmSupport;
use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic};

use super::native_class::{LgtNativeClass, parse_native_class};
use crate::runtime::SVC_CATEGORY_JAVA_TRAMPOLINE;

const OBJ_HEADER_SIZE: u32 = 0x0c;
const OBJ_PTR_FIELDS_OFFSET: u32 = 0x08;
const FIELD_ARRAY_WORDS: u32 = 256;
/// Per-class vtable size (words). Imported virtual-method global indices are small
/// (< ~30); this covers them with margin.
const VTABLE_WORDS: u32 = 64;

// ---- shared runtime ----

/// Process-wide LGT JVM glue, shared (cheap `Arc` clones) between class
/// definitions, the trampoline SVC handler, and `java_load_classes`.
#[derive(Clone)]
pub struct LgtJvmShared {
    pub jvm: Jvm,
    #[allow(dead_code)] // kept for parity / future platform-service access
    pub system: System,
    /// guest object pointer -> its JVM instance.
    instances: Arc<Mutex<BTreeMap<u32, Box<dyn ClassInstance>>>>,
    /// native -> platform method trampolines, indexed by SVC id.
    trampolines: Arc<Mutex<Vec<TrampEntry>>>,
    /// Base of the global virtual-method offset table (`java_load_classes` output).
    /// Used as the vtable word for **app** objects, which extend the lcdui hierarchy
    /// and so dispatch through the union of all imported lcdui-hierarchy methods.
    vmethod_table: Arc<Mutex<u32>>,
    /// Per-platform-class vtable base (only that class's own imported virtual methods
    /// at their global indices, everything else 0). A **platform proxy** object uses
    /// its class's vtable so an index that belongs to another class (e.g. Graphics'
    /// `drawLine`@14) does not misfire on, say, a `Runtime` — it reads 0 instead.
    class_vtables: Arc<Mutex<BTreeMap<String, u32>>>,
}

impl LgtJvmShared {
    pub fn new(jvm: Jvm, system: System) -> Self {
        Self {
            jvm,
            system,
            instances: Arc::new(Mutex::new(BTreeMap::new())),
            trampolines: Arc::new(Mutex::new(Vec::new())),
            vmethod_table: Arc::new(Mutex::new(0)),
            class_vtables: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    fn register_instance(&self, guest_ptr: u32, instance: Box<dyn ClassInstance>) {
        self.instances.lock().insert(guest_ptr, instance);
    }

    /// Object `+0x00` value: the virtual-method table base (for AOT vtable dispatch).
    fn vtable_word(&self) -> u32 {
        *self.vmethod_table.lock()
    }

    /// Map a JVM value to the guest word the native code expects (`this`/args).
    /// Object values become a guest pointer; a platform object with no guest
    /// backing yet gets a freshly-allocated proxy block.
    fn value_to_guest(&self, core: &mut ArmCore, value: &JavaValue) -> u32 {
        match value {
            JavaValue::Void => 0,
            JavaValue::Boolean(x) => *x as u32,
            JavaValue::Byte(x) => *x as i32 as u32,
            JavaValue::Char(x) => *x as u32,
            JavaValue::Short(x) => *x as i32 as u32,
            JavaValue::Int(x) => *x as u32,
            JavaValue::Float(x) => x.to_bits(),
            JavaValue::Long(x) => *x as u32,
            JavaValue::Double(x) => x.to_bits() as u32,
            JavaValue::Object(None) => 0,
            JavaValue::Object(Some(inst)) => {
                if let Some(o) = inst.as_any().downcast_ref::<LgtClassInstance>() {
                    return o.guest_ptr;
                }
                // platform object: allocate an opaque proxy block whose vtable word
                // is its own class's per-class vtable (so foreign indices read 0),
                // and register it.
                let class_name = inst.class_definition().name();
                let vtable = self.class_vtables.lock().get(&class_name).copied().unwrap_or_else(|| self.vtable_word());
                match Allocator::alloc(core, OBJ_HEADER_SIZE) {
                    Ok(ptr) => {
                        let _ = write_generic(core, ptr, vtable);
                        let _ = write_generic(core, ptr + 4, 0u32);
                        let _ = write_generic(core, ptr + OBJ_PTR_FIELDS_OFFSET, 0u32);
                        self.instances.lock().insert(ptr, inst.clone());
                        ptr
                    }
                    Err(_) => 0,
                }
            }
        }
    }

    /// Map a guest word back to a JVM value of the given type (args/returns from
    /// native into platform methods).
    fn guest_to_value(&self, raw: u32, ty: &JavaType) -> JavaValue {
        match ty {
            JavaType::Void => JavaValue::Void,
            JavaType::Boolean => JavaValue::Boolean(raw != 0),
            JavaType::Byte => JavaValue::Byte(raw as i8),
            JavaType::Char => JavaValue::Char(raw as u16),
            JavaType::Short => JavaValue::Short(raw as i16),
            JavaType::Int => JavaValue::Int(raw as i32),
            JavaType::Float => JavaValue::Float(f32::from_bits(raw)),
            JavaType::Long => JavaValue::Long(raw as i64),
            JavaType::Double => JavaValue::Double(f64::from_bits(raw as u64)),
            JavaType::Class(_) | JavaType::Array(_) => {
                if raw == 0 {
                    JavaValue::Object(None)
                } else {
                    JavaValue::Object(self.instances.lock().get(&raw).cloned())
                }
            }
            _ => JavaValue::Void,
        }
    }
}

// ---- class metadata (pure Rust) ----

#[derive(Clone, Debug)]
struct MethodMeta {
    name: String,
    descriptor: String,
    access_flags: MethodAccessFlags,
    code_ptr: u32,
}

#[derive(Clone, Debug)]
struct FieldMeta {
    name: String,
    descriptor: String,
    access_flags: FieldAccessFlags,
}

#[derive(Clone)]
pub struct LgtClassDefinition {
    inner: Arc<ClassInner>,
}

struct ClassInner {
    name: String,
    super_name: Option<String>,
    methods: Vec<MethodMeta>,
    fields: Vec<FieldMeta>,
    statics: Mutex<BTreeMap<String, JavaValue>>,
    core: ArmCore,
    shared: LgtJvmShared,
}

impl LgtClassDefinition {
    fn from_native(class: &LgtNativeClass, core: ArmCore, shared: LgtJvmShared) -> Self {
        let methods = class
            .methods
            .iter()
            .map(|m| MethodMeta {
                name: m.name.clone(),
                descriptor: m.signature.clone(),
                access_flags: MethodAccessFlags::from_bits_truncate(m.access_flags as u16),
                code_ptr: m.code_ptr,
            })
            .collect();
        let fields = class
            .fields
            .iter()
            .map(|f| FieldMeta {
                name: f.name.clone(),
                descriptor: f.type_descriptor.clone(),
                access_flags: FieldAccessFlags::from_bits_truncate(f.access_flags as u16),
            })
            .collect();

        Self {
            inner: Arc::new(ClassInner {
                name: class.name.clone(),
                super_name: class.parent_name.clone(),
                methods,
                fields,
                statics: Mutex::new(BTreeMap::new()),
                core,
                shared,
            }),
        }
    }
}

impl Debug for LgtClassDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "LgtClassDefinition({})", self.inner.name)
    }
}

#[async_trait::async_trait]
impl ClassDefinition for LgtClassDefinition {
    fn name(&self) -> String {
        self.inner.name.clone()
    }
    fn super_class_name(&self) -> Option<String> {
        self.inner.super_name.clone()
    }
    fn access_flags(&self) -> ClassAccessFlags {
        ClassAccessFlags::PUBLIC
    }

    async fn instantiate(&self, jvm: &Jvm) -> JvmResult<Box<dyn ClassInstance>> {
        let mut core = self.inner.core.clone();

        let vtable_word = self.inner.shared.vtable_word();
        let alloc = (|| -> Result<u32> {
            let ptr_fields = Allocator::alloc(&mut core, FIELD_ARRAY_WORDS * 4)?;
            wie_util::ByteWrite::write_bytes(&mut core, ptr_fields, &[0u8; (FIELD_ARRAY_WORDS * 4) as usize])?;
            let ptr_raw = Allocator::alloc(&mut core, OBJ_HEADER_SIZE)?;
            write_generic(&mut core, ptr_raw, vtable_word)?; // +0: virtual-method table base
            write_generic(&mut core, ptr_raw + 4, 0u32)?;
            write_generic(&mut core, ptr_raw + OBJ_PTR_FIELDS_OFFSET, ptr_fields)?;
            Ok(ptr_raw)
        })();
        let ptr_raw = match alloc {
            Ok(p) => p,
            Err(e) => return Err(jvm.exception("java/lang/OutOfMemoryError", &e.to_string()).await),
        };

        let instance = LgtClassInstance {
            guest_ptr: ptr_raw,
            core,
            definition: self.clone(),
            jvm_fields: Arc::new(Mutex::new(BTreeMap::new())),
        };
        self.inner.shared.register_instance(ptr_raw, Box::new(instance.clone()));

        tracing::trace!("LGT instantiate {} -> guest {ptr_raw:#x}", self.inner.name);
        Ok(Box::new(instance))
    }

    fn method(&self, name: &str, descriptor: &str, _is_static: bool) -> Option<Box<dyn Method>> {
        self.inner.methods.iter().find(|m| m.name == name && m.descriptor == descriptor).map(|m| {
            Box::new(LgtMethod {
                class_name: self.inner.name.clone(),
                meta: m.clone(),
                core: self.inner.core.clone(),
                shared: self.inner.shared.clone(),
            }) as Box<dyn Method>
        })
    }

    fn field(&self, name: &str, descriptor: &str, _is_static: bool) -> Option<Box<dyn Field>> {
        self.inner
            .fields
            .iter()
            .find(|f| f.name == name && f.descriptor == descriptor)
            .map(|f| Box::new(LgtField { meta: f.clone() }) as Box<dyn Field>)
    }

    fn fields(&self) -> Vec<Box<dyn Field>> {
        self.inner
            .fields
            .iter()
            .map(|f| Box::new(LgtField { meta: f.clone() }) as Box<dyn Field>)
            .collect()
    }

    fn get_static_field(&self, field: &dyn Field) -> JvmResult<JavaValue> {
        let key = field_key(&field.name(), &field.descriptor());
        Ok(self
            .inner
            .statics
            .lock()
            .get(&key)
            .cloned()
            .unwrap_or_else(|| JavaType::parse(&field.descriptor()).default()))
    }
    fn put_static_field(&mut self, field: &dyn Field, value: JavaValue) -> JvmResult<()> {
        self.inner.statics.lock().insert(field_key(&field.name(), &field.descriptor()), value);
        Ok(())
    }
}

// ---- instance ----

#[derive(Clone)]
pub struct LgtClassInstance {
    guest_ptr: u32,
    #[allow(dead_code)] // for guest-memory field unification (item 4, partial)
    core: ArmCore,
    definition: LgtClassDefinition,
    // JVM-side field storage. TODO(cp3 item 4): unify with the guest field array so
    // native-written and JVM-read fields agree (needs the field-offset slot map).
    jvm_fields: Arc<Mutex<BTreeMap<String, JavaValue>>>,
}

impl Debug for LgtClassInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{:#x}", self.definition.inner.name, self.guest_ptr)
    }
}
impl Hash for LgtClassInstance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.guest_ptr.hash(state)
    }
}

#[async_trait::async_trait]
impl ClassInstance for LgtClassInstance {
    fn destroy(self: Box<Self>) {}
    fn class_definition(&self) -> Box<dyn ClassDefinition> {
        Box::new(self.definition.clone())
    }
    fn equals(&self, other: &dyn ClassInstance) -> JvmResult<bool> {
        Ok(other
            .as_any()
            .downcast_ref::<LgtClassInstance>()
            .map(|o| o.guest_ptr == self.guest_ptr)
            .unwrap_or(false))
    }
    fn get_field(&self, field: &dyn Field) -> JvmResult<JavaValue> {
        let key = field_key(&field.name(), &field.descriptor());
        Ok(self
            .jvm_fields
            .lock()
            .get(&key)
            .cloned()
            .unwrap_or_else(|| JavaType::parse(&field.descriptor()).default()))
    }
    fn put_field(&mut self, field: &dyn Field, value: JavaValue) -> JvmResult<()> {
        self.jvm_fields.lock().insert(field_key(&field.name(), &field.descriptor()), value);
        Ok(())
    }
}

// ---- field / method ----

#[derive(Debug)]
struct LgtField {
    meta: FieldMeta,
}
impl Field for LgtField {
    fn name(&self) -> String {
        self.meta.name.clone()
    }
    fn descriptor(&self) -> String {
        self.meta.descriptor.clone()
    }
    fn access_flags(&self) -> FieldAccessFlags {
        self.meta.access_flags
    }
}

struct LgtMethod {
    class_name: String,
    meta: MethodMeta,
    core: ArmCore,
    shared: LgtJvmShared,
}
impl Debug for LgtMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "LgtMethod({}.{}{})", self.class_name, self.meta.name, self.meta.descriptor)
    }
}

#[async_trait::async_trait]
impl Method for LgtMethod {
    fn name(&self) -> String {
        self.meta.name.clone()
    }
    fn descriptor(&self) -> String {
        self.meta.descriptor.clone()
    }
    fn access_flags(&self) -> MethodAccessFlags {
        self.meta.access_flags
    }

    async fn run(&self, _jvm: &Jvm, args: Box<[JavaValue]>) -> JvmResult<JavaValue> {
        let mut core = self.core.clone();
        let params: Vec<u32> = args.iter().map(|v| self.shared.value_to_guest(&mut core, v)).collect();

        tracing::debug!(
            "LGT dispatch -> native {}.{}{} code={:#x} params={:x?}",
            self.class_name,
            self.meta.name,
            self.meta.descriptor,
            self.meta.code_ptr,
            params
        );

        let r0: u32 = match core.run_function(self.meta.code_ptr, &params).await {
            Ok(r) => r,
            Err(e) => {
                let msg = format!(
                    "native dispatch {}.{}{} @ {:#x}: {e}",
                    self.class_name, self.meta.name, self.meta.descriptor, self.meta.code_ptr
                );
                return Err(self.shared.jvm.exception("java/lang/Error", &msg).await);
            }
        };

        let ret = match JavaType::parse(&self.meta.descriptor) {
            JavaType::Method(_, ret) => *ret,
            _ => JavaType::Void,
        };
        Ok(self.shared.guest_to_value(r0, &ret))
    }
}

fn field_key(name: &str, descriptor: &str) -> String {
    format!("{name}:{descriptor}")
}

// ---- native -> platform trampolines ----

#[derive(Clone)]
struct TrampEntry {
    class_name: String,
    name: String,
    descriptor: String,
    is_virtual: bool,
}

/// SVC handler for a native -> platform call. The SVC id selects the trampoline;
/// `r0..` carry `this`(virtual/<init>) + args; the result goes back in `r0`.
pub async fn handle_java_trampoline(core: &mut ArmCore, shared: &mut LgtJvmShared, id: SvcId) -> Result<u32> {
    let (_, lr) = core.read_pc_lr()?;
    let entry = {
        let table = shared.trampolines.lock();
        match table.get(id.0 as usize) {
            Some(e) => e.clone(),
            None => return Err(WieError::FatalError(format!("LGT trampoline: unknown id {}", id.0))),
        }
    };

    // A null/placeholder slot the app declared but does not implement: no-op.
    if entry.name.is_empty() {
        tracing::trace!("LGT trampoline noop slot {}", id.0);
        core.set_next_pc(lr)?;
        return Ok(0);
    }

    let arg_types = match JavaType::parse(&entry.descriptor) {
        JavaType::Method(a, _) => a,
        _ => Vec::new(),
    };

    // `this` first for virtual/<init>; then one guest word per arg slot.
    let is_static = !entry.is_virtual && entry.name != "<init>";
    let mut pos = 0usize;
    let this = if is_static {
        None
    } else {
        let raw = core.read_param(pos)?;
        pos += 1;
        shared.instances.lock().get(&raw).cloned()
    };
    let mut jargs = Vec::with_capacity(arg_types.len());
    for ty in &arg_types {
        let raw = core.read_param(pos)?;
        pos += 1;
        if matches!(ty, JavaType::Long | JavaType::Double) {
            pos += 1; // 64-bit args take two slots (low word used)
        }
        jargs.push(shared.guest_to_value(raw, ty));
    }

    let this_class = this.as_ref().map(|t| t.class_definition().name());
    tracing::debug!(
        "LGT trampoline id={} -> {}.{}{}  this_actual={:?}",
        id.0,
        entry.class_name,
        entry.name,
        entry.descriptor,
        this_class
    );

    let jvm = shared.jvm.clone();
    let result: core::result::Result<JavaValue, JavaError> = if entry.name == "<init>" {
        match &this {
            Some(this) => jvm.invoke_special(this, &entry.class_name, "<init>", &entry.descriptor, jargs).await,
            None => Err(jvm.exception("java/lang/NullPointerException", "<init> without this").await),
        }
    } else if is_static {
        jvm.invoke_static(&entry.class_name, &entry.name, &entry.descriptor, jargs).await
    } else {
        match &this {
            Some(this) => jvm.invoke_virtual(this, &entry.name, &entry.descriptor, jargs).await,
            None => Err(jvm.exception("java/lang/NullPointerException", &entry.name).await),
        }
    };

    // No no-op fallback (it diverges — confirmed: a 0 return for getHeight loops the
    // layout). Unresolved calls fail loudly so the blocker stays visible.
    let result = match result {
        Ok(v) => v,
        Err(e) => return Err(JvmSupport::to_wie_err(&jvm, e).await),
    };

    let r0 = shared.value_to_guest(core, &result);
    core.set_next_pc(lr)?;
    Ok(r0)
}

// ---- registration + table install ----

pub fn register_java_trampoline_handler(core: &mut ArmCore, shared: &LgtJvmShared) -> Result<()> {
    core.register_svc_handler(SVC_CATEGORY_JAVA_TRAMPOLINE, handle_java_trampoline, shared)
}

/// Empirically-identified vtable slots for `java/lang/*` classes whose layout is
/// NOT in the app's import data (they declare 0 imported virtual methods) but which
/// the AOT calls by hardcoded vtable index. Each entry is `(vtable_index, name,
/// descriptor)`. These are **estimates (추정)** grounded in how the native code uses
/// the call (see STEP report's evidence table), not a derived spec — extend as more
/// (class, index) pairs are observed.
fn known_java_lang_vtable(class: &str) -> &'static [(u32, &'static str, &'static str)] {
    match class {
        // Game.<init> startup: getRuntime().<14>() result discarded (void => gc),
        // then getRuntime().<13>() result used as a value (=> freeMemory).
        "java/lang/Runtime" => &[(13, "freeMemory", "()J"), (14, "gc", "()V")],
        _ => &[],
    }
}

fn read_pair(core: &ArmCore, base: u32, idx: u32) -> (Option<String>, Option<String>) {
    let n = read_generic::<u32, _>(core, base + idx * 8).unwrap_or(0);
    let t = read_generic::<u32, _>(core, base + idx * 8 + 4).unwrap_or(0);
    (read_cstr(core, n), read_cstr(core, t))
}

fn read_cstr(core: &ArmCore, ptr: u32) -> Option<String> {
    if ptr == 0 {
        return None;
    }
    let bytes = read_null_terminated_string_bytes(core, ptr).ok()?;
    if bytes.is_empty() || !bytes.iter().all(|&b| (0x20..0x7f).contains(&b)) {
        return None;
    }
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

/// Implement `java_load_classes`: for each imported platform class, resolve its
/// requested virtual/static methods to trampolines and write them into the
/// fixed-offset output tables the native code reads.
#[allow(clippy::too_many_arguments)]
pub fn install_platform_tables(
    core: &mut ArmCore,
    shared: &LgtJvmShared,
    classes: u32,
    virtual_methods: u32,
    static_methods: u32,
    field_offsets: u32,
    virtual_method_offsets: u32,
    static_method_offsets: u32,
) -> Result<()> {
    let count = read_generic::<u32, _>(core, classes).unwrap_or(0);
    tracing::debug!("install_platform_tables: {count} imported classes");

    // The AOT code's per-object vtable pointer (`this+0`) indexes this global table.
    *shared.vmethod_table.lock() = virtual_method_offsets;

    let mut method_slots = 0usize;
    let mut field_slots = 0usize;

    for i in 0..count {
        let base = classes + 4 + i * 24;
        let name = match read_cstr(core, read_generic::<u32, _>(core, base).unwrap_or(0)) {
            Some(n) => n,
            None => continue,
        };
        let vmo = read_generic::<u16, _>(core, base + 12).unwrap_or(0) as u32;
        let vmc = read_generic::<u16, _>(core, base + 14).unwrap_or(0) as u32;
        let sfo = read_generic::<u16, _>(core, base + 8).unwrap_or(0) as u32;
        let sfc = read_generic::<u16, _>(core, base + 10).unwrap_or(0) as u32;
        let smo = read_generic::<u16, _>(core, base + 20).unwrap_or(0) as u32;
        let smc = read_generic::<u16, _>(core, base + 22).unwrap_or(0) as u32;

        // Per-class vtable: a fresh, zeroed array; the class's own virtual methods
        // are placed at their global indices. A platform proxy of this class points
        // here, so an index belonging to another class reads 0 (a clean null) instead
        // of a foreign trampoline. Created for every imported class — even those with
        // 0 virtual methods (e.g. Runtime), whose empty vtable makes an unresolvable
        // index fail cleanly rather than misfire onto another class's method.
        let class_vtable = Allocator::alloc(core, VTABLE_WORDS * 4)?;
        wie_util::ByteWrite::write_bytes(core, class_vtable, &[0u8; (VTABLE_WORDS * 4) as usize])?;
        shared.class_vtables.lock().insert(name.clone(), class_vtable);

        for j in 0..vmc {
            let idx = vmo + j;
            let (mname, mtype) = read_pair(core, virtual_methods, idx);
            let stub = make_method_trampoline(core, shared, &name, mname, mtype, true)?;
            write_generic(core, virtual_method_offsets + idx * 4, stub)?;
            if idx < VTABLE_WORDS {
                write_generic(core, class_vtable + idx * 4, stub)?;
            }
            method_slots += 1;
        }
        for j in 0..smc {
            let idx = smo + j;
            let (mname, mtype) = read_pair(core, static_methods, idx);
            let stub = make_method_trampoline(core, shared, &name, mname, mtype, false)?;
            write_generic(core, static_method_offsets + idx * 4, stub)?;
            method_slots += 1;
        }

        // `java/lang/*` classes declare 0 imported virtual methods, yet the AOT calls
        // them by hardcoded vtable index (their layout isn't in the app data — see
        // STEP report). Fill the few slots identified empirically (from how the AOT
        // uses the result) so the boot can get past the startup memory check. These
        // are 추정 (estimated) placements, marked as such.
        for &(idx, mname, mtype) in known_java_lang_vtable(&name) {
            let stub = make_method_trampoline(core, shared, &name, Some(mname.into()), Some(mtype.into()), true)?;
            if idx < VTABLE_WORDS {
                write_generic(core, class_vtable + idx * 4, stub)?;
            }
            method_slots += 1;
        }
        // Fields: write a distinct, non-colliding guest slot per field so native
        // field accesses don't all alias slot 0. (Full JVM<->native unification is
        // tracked as item 4.)
        for j in 0..sfc {
            let idx = sfo + j;
            let slot = (idx % FIELD_ARRAY_WORDS) as u16;
            write_generic(core, field_offsets + idx * 2, slot)?;
            field_slots += 1;
        }
    }

    tracing::info!("LGT java_load_classes: filled {method_slots} method slots, {field_slots} field slots");
    Ok(())
}

fn make_method_trampoline(
    core: &mut ArmCore,
    shared: &LgtJvmShared,
    class_name: &str,
    mname: Option<String>,
    mtype: Option<String>,
    is_virtual: bool,
) -> Result<u32> {
    let entry = match (mname, mtype) {
        (Some(name), Some(descriptor)) => TrampEntry {
            class_name: class_name.to_string(),
            name,
            descriptor,
            is_virtual,
        },
        // Declared-but-unnamed slot: still callable, as a no-op.
        _ => TrampEntry {
            class_name: class_name.to_string(),
            name: String::new(),
            descriptor: String::new(),
            is_virtual,
        },
    };
    let id = {
        let mut table = shared.trampolines.lock();
        table.push(entry);
        (table.len() - 1) as u32
    };
    core.make_svc_stub(SVC_CATEGORY_JAVA_TRAMPOLINE, id)
}

// ---- app class scan + registration (unchanged structure) ----

fn scan_class_headers(core: &ArmCore, data_start: u32, data_end: u32) -> Vec<u32> {
    let in_data = |v: u32| v >= data_start && v < data_end;
    let is_short_name = |ptr: u32| -> bool {
        if ptr == 0 {
            return false;
        }
        match read_null_terminated_string_bytes(core, ptr) {
            Ok(b) => !b.is_empty() && b.len() <= 24 && b.iter().all(|&c| (0x20..0x7f).contains(&c)),
            Err(_) => false,
        }
    };
    let small_count = |table: u32| -> bool {
        if table == 0 {
            return true;
        }
        if !in_data(table) {
            return false;
        }
        matches!(read_generic::<u32, _>(core, table), Ok(c) if c < 512)
    };

    let mut out = Vec::new();
    let mut va = data_start;
    while va + 0x40 <= data_end {
        let read = |off: u32| read_generic::<u32, _>(core, va + off).unwrap_or(0);
        let tag = read(0);
        let parent_ok = read(0x10) == 0 || in_data(read(0x10)) || is_short_name(read(0x10));
        if tag > 0 && tag < 0x1000 && is_short_name(read(0x08)) && parent_ok && small_count(read(0x38)) && small_count(read(0x3c)) {
            out.push(va);
        }
        va += 4;
    }
    out
}

/// Scan the app's `.data` for native class headers and register each as an
/// ARM-backed JVM class. No-op (empty) when none are found (clet path unaffected).
pub async fn register_app_classes(jvm: &Jvm, core: &mut ArmCore, shared: &LgtJvmShared, data_start: u32, data_end: u32) -> Result<Vec<String>> {
    let headers = scan_class_headers(core, data_start, data_end);
    if headers.is_empty() {
        return Ok(Vec::new());
    }
    tracing::debug!("LGT native JVM: found {} app class headers in .data", headers.len());

    let mut pending: Vec<LgtNativeClass> = Vec::new();
    let mut seen = BTreeSet::new();
    for header in headers {
        if let Ok(class) = parse_native_class(core, header)
            && !class.name.is_empty()
            && seen.insert(class.name.clone())
        {
            pending.push(class);
        }
    }
    let app_names: BTreeSet<String> = pending.iter().map(|c| c.name.clone()).collect();

    let mut registered = Vec::new();
    let mut done = BTreeSet::new();
    loop {
        let mut progressed = false;
        let mut still = Vec::new();
        for class in pending {
            let parent_ready = match &class.parent_name {
                Some(p) => !app_names.contains(p) || done.contains(p),
                None => true,
            };
            if !parent_ready {
                still.push(class);
                continue;
            }
            let name = class.name.clone();
            let definition = LgtClassDefinition::from_native(&class, core.clone(), shared.clone());
            match jvm.register_class(Box::new(definition), None).await {
                Ok(_) => {
                    tracing::debug!("LGT native JVM: registered {name:?} (parent={:?})", class.parent_name);
                    done.insert(name.clone());
                    registered.push(name);
                    progressed = true;
                }
                Err(e) => tracing::warn!("LGT native JVM: failed to register {name:?}: {e:?}"),
            }
        }
        pending = still;
        if pending.is_empty() || !progressed {
            break;
        }
    }
    if !pending.is_empty() {
        let names: Vec<&String> = pending.iter().map(|c| &c.name).collect();
        tracing::warn!("LGT native JVM: {} classes left unregistered: {names:?}", pending.len());
    }
    Ok(registered)
}
