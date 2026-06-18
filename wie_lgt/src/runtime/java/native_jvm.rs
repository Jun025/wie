//! LGT native-backed JVM: registers an AOT-compiled app's native classes (decoded
//! by [`super::native_class`]) as real JVM classes whose method bodies dispatch to
//! the app's ARM code.
//!
//! Design note: this mirrors `wie_ktf`'s `jvm_support`, but LGT keeps its own class
//! metadata in `.data` (see `docs/lgt_native_classes.md`) rather than reflecting JVM
//! metadata into guest memory — so we only need (a) JVM `ClassDefinition`s built from
//! the parsed descriptors and (b) method bodies that marshal to/from ARM. The
//! approach is under discussion with the maintainer, so it is kept in its own module.
//!
//! Checkpoint status (see STEP2_REPORT.md):
//!  - C1: classes registered, method bodies are logging stubs (Game instantiation).
//!  - C2+: real ARM dispatch — not yet wired (`dispatch` is a stub).

use alloc::{boxed::Box, collections::BTreeSet, string::String, vec::Vec};

use java_class_proto::{JavaFieldProto, JavaMethodProto, MethodBody};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use jvm::{JavaError, JavaValue, Jvm};
use jvm_rust::{ClassDefinitionImpl, FieldImpl, MethodImpl};

use wie_backend::System;
use wie_core_arm::ArmCore;
use wie_util::Result;

use super::native_class::{LgtNativeClass, parse_native_class};

/// Context handed to every native method body: enough to dispatch into ARM.
#[derive(Clone)]
#[allow(dead_code)] // `core`/`system` are consumed by ARM dispatch in a later checkpoint
pub struct LgtClassContext {
    pub core: ArmCore,
    pub system: System,
}

/// A JVM method whose implementation is native ARM code at `code_ptr`.
struct LgtNativeMethodBody {
    class_name: String,
    name: String,
    descriptor: String,
    parent_name: Option<String>,
    code_ptr: u32,
    is_static: bool,
}

#[async_trait::async_trait]
impl MethodBody<JavaError, LgtClassContext> for LgtNativeMethodBody {
    async fn call(&self, jvm: &Jvm, _context: &mut LgtClassContext, args: Box<[JavaValue]>) -> core::result::Result<JavaValue, JavaError> {
        // Real ARM dispatch (marshal args -> r0..r3, run_function(code_ptr), marshal
        // the return value) requires the app's objects to be ARM-memory-backed so
        // `this`/object args can be passed as guest pointers — i.e. a custom
        // ARM-backed ClassInstance, like wie_ktf's jvm_support. That object-model
        // port is the pending structural decision; until it lands, method bodies
        // are stubs (see STEP2_REPORT.md).
        //
        // Interim: chain a parameterless `<init>` to its superclass constructor so
        // the platform `org/kwis/msp/lcdui/Jlet` machinery initialises (registers the
        // current Jlet, creates Display/EventQueue). This lets the boot proceed past
        // `WIPIMIDlet.startApp` to the app's `startApp` without running native code.
        if self.name == "<init>"
            && self.descriptor == "()V"
            && let (Some(parent), Some(this_val @ JavaValue::Object(Some(_)))) = (&self.parent_name, args.first())
        {
            tracing::debug!("LGT native <init> chain: {}.<init> -> {parent}.<init> (interim)", self.class_name);
            let this: Box<dyn jvm::ClassInstance> = this_val.clone().into();
            let _: () = jvm.invoke_special(&this, parent, "<init>", "()V", ()).await?;
            return Ok(JavaValue::Void);
        }

        tracing::warn!(
            "LGT native dispatch stub: {}.{}{} code={:#x} static={} (returning default)",
            self.class_name,
            self.name,
            self.descriptor,
            self.code_ptr,
            self.is_static
        );

        Ok(default_return_value(&self.descriptor))
    }
}

/// Default `JavaValue` for a method descriptor's return type (`(...)R`).
fn default_return_value(descriptor: &str) -> JavaValue {
    let ret = descriptor.rsplit(')').next().unwrap_or("V");
    match ret.chars().next().unwrap_or('V') {
        'V' => JavaValue::Void,
        'Z' => JavaValue::Boolean(false),
        'B' => JavaValue::Byte(0),
        'C' => JavaValue::Char(0),
        'S' => JavaValue::Short(0),
        'I' => JavaValue::Int(0),
        'J' => JavaValue::Long(0),
        'F' => JavaValue::Float(0.0),
        'D' => JavaValue::Double(0.0),
        'L' | '[' => JavaValue::Object(None),
        _ => JavaValue::Void,
    }
}

/// Build a JVM `ClassDefinitionImpl` from a parsed native class. Method bodies are
/// [`LgtNativeMethodBody`]; the parent is the resolved (obfuscated or platform) name.
fn build_class_definition(class: &LgtNativeClass, context: LgtClassContext) -> ClassDefinitionImpl {
    let methods: Vec<MethodImpl> = class
        .methods
        .iter()
        .map(|m| {
            let access_flags = MethodAccessFlags::from_bits_truncate(m.access_flags as u16);
            let is_static = access_flags.contains(MethodAccessFlags::STATIC);
            let proto = JavaMethodProto {
                name: m.name.clone(),
                descriptor: m.signature.clone(),
                body: Box::new(LgtNativeMethodBody {
                    class_name: class.name.clone(),
                    name: m.name.clone(),
                    descriptor: m.signature.clone(),
                    parent_name: class.parent_name.clone(),
                    code_ptr: m.code_ptr,
                    is_static,
                }) as Box<dyn MethodBody<_, _>>,
                access_flags,
            };
            MethodImpl::from_method_proto(proto, Box::new(context.clone()) as Box<_>)
        })
        .collect();

    let fields: Vec<FieldImpl> = class
        .fields
        .iter()
        .map(|f| {
            let access_flags = FieldAccessFlags::from_bits_truncate(f.access_flags as u16);
            FieldImpl::from_field_proto(JavaFieldProto::new(&f.name, &f.type_descriptor, access_flags))
        })
        .collect();

    // Keep class flags minimal (PUBLIC) so instantiation is never blocked by a
    // spuriously-decoded ABSTRACT/INTERFACE bit; refine in a later checkpoint.
    ClassDefinitionImpl::new(&class.name, class.parent_name.clone(), ClassAccessFlags::PUBLIC, methods, fields)
}

/// Heuristic class-header detector over the app's `.data` segment, matching the
/// layout documented in `docs/lgt_native_classes.md`: a record whose `+0x08` is a
/// short cstring (class name), `+0x10` is 0 / a cstring / a `.data` pointer
/// (parent), and `+0x38`/`+0x3c` are 0 / `.data` count-prefixed tables.
fn scan_class_headers(core: &ArmCore, data_start: u32, data_end: u32) -> Vec<u32> {
    use wie_util::{read_generic, read_null_terminated_string_bytes};

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
        let ptr_name = read(0x08);
        let ptr_parent = read(0x10);
        let ptr_methods = read(0x38);
        let ptr_fields = read(0x3c);

        let parent_ok = ptr_parent == 0 || in_data(ptr_parent) || is_short_name(ptr_parent);
        if tag > 0 && tag < 0x1000 && is_short_name(ptr_name) && parent_ok && small_count(ptr_methods) && small_count(ptr_fields) {
            out.push(va);
        }
        va += 4;
    }

    out
}

/// Scan the app's `.data` for native class headers and register each as a JVM class.
/// Read-only w.r.t. guest memory; returns the registered class names. No-op (returns
/// empty) when no headers are found — so the WIPI-C clet path is unaffected.
pub async fn register_app_classes(jvm: &Jvm, core: &mut ArmCore, system: &System, data_start: u32, data_end: u32) -> Result<Vec<String>> {
    let headers = scan_class_headers(core, data_start, data_end);
    if headers.is_empty() {
        return Ok(Vec::new());
    }

    tracing::debug!("LGT native JVM: found {} app class headers in .data", headers.len());

    let context = LgtClassContext {
        core: core.clone(),
        system: system.clone(),
    };

    // Parse all classes (dedupe by name), then register in dependency order:
    // `register_class` resolves the superclass eagerly, so an app-class parent must
    // be registered first (e.g. Game -> a, b -> o). App parents not yet registered
    // are deferred; platform parents resolve via the bootstrap loader.
    let mut pending: Vec<LgtNativeClass> = Vec::new();
    let mut seen_names = BTreeSet::new();
    for header in headers {
        if let Ok(class) = parse_native_class(core, header)
            && !class.name.is_empty()
            && seen_names.insert(class.name.clone())
        {
            pending.push(class);
        }
    }
    let app_names: BTreeSet<String> = pending.iter().map(|c| c.name.clone()).collect();

    let mut registered = Vec::new();
    let mut done = BTreeSet::new();
    loop {
        let mut progressed = false;
        let mut still_pending = Vec::new();
        for class in pending {
            let parent_ready = match &class.parent_name {
                // platform parent (not one of ours) -> resolvable via bootstrap loader
                Some(p) => !app_names.contains(p) || done.contains(p),
                None => true,
            };
            if !parent_ready {
                still_pending.push(class);
                continue;
            }

            let name = class.name.clone();
            let definition = build_class_definition(&class, context.clone());
            match jvm.register_class(Box::new(definition), None).await {
                Ok(_) => {
                    tracing::debug!(
                        "LGT native JVM: registered {:?} (parent={:?}, {} methods, {} fields)",
                        name,
                        class.parent_name,
                        class.methods.len(),
                        class.fields.len()
                    );
                    done.insert(name.clone());
                    registered.push(name);
                    progressed = true;
                }
                Err(e) => tracing::warn!("LGT native JVM: failed to register {name:?}: {e:?}"),
            }
        }
        pending = still_pending;
        if pending.is_empty() || !progressed {
            break;
        }
    }
    if !pending.is_empty() {
        let names: Vec<&String> = pending.iter().map(|c| &c.name).collect();
        tracing::warn!(
            "LGT native JVM: {} classes left unregistered (parent cycle/missing): {names:?}",
            pending.len()
        );
    }

    Ok(registered)
}
