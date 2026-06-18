//! LGT native-backed JVM (PoC): registers an AOT-compiled app's native classes
//! (decoded by [`super::native_class`]) as real JVM classes whose instances are
//! backed by guest (ARM) memory and whose methods dispatch to the app's ARM code.
//!
//! Design (agreed in Discussion #1232 — keep LGT-specific, don't over-engineer):
//!  - Each app instance is a guest object block; `this+0x08` points to a zeroed
//!    field array, matching what the AOT code expects (`r1=[this,#8]; str rX,
//!    [r1, idx<<2]`). This is the minimal layout needed for native bodies to run.
//!  - JVM-side field get/put use a separate Rust map (sufficient for the platform
//!    `Jlet`/`Display` glue that touches inherited fields). Unifying the two stores
//!    needs the platform field-offset table — that is checkpoint 3.
//!  - Method dispatch marshals `this`+args into `r0..r3`, `run_function(code_ptr)`,
//!    and marshals the return per the descriptor.
//!
//! Scope (checkpoint 2): the app's OWN methods dispatch to real ARM. Calls into
//! platform classes go through the `java_load_classes` method/offset tables
//! (`.bss` @ `0x1500xxx`), which are not yet filled — that is checkpoint 3.

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
use jvm::{ClassDefinition, ClassInstance, Field, JavaType, JavaValue, Jvm, Method, Result as JvmResult};
use spin::Mutex;

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore};
use wie_util::{Result, read_generic, write_generic};

use super::native_class::{LgtNativeClass, parse_native_class};

/// `this+0x08` holds the pointer to the instance's field array.
const OBJ_HEADER_SIZE: u32 = 0x0c;
const OBJ_PTR_FIELDS_OFFSET: u32 = 0x08;
/// Generous field-array size (words). The AOT code indexes fields by slot; until
/// the platform offset table is wired (cp3) most writes land near slot 0.
const FIELD_ARRAY_WORDS: u32 = 128;

// ---- metadata (pure Rust; no guest reflection, unlike wie_ktf) ----

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
    access_flags: ClassAccessFlags,
    methods: Vec<MethodMeta>,
    fields: Vec<FieldMeta>,
    statics: Mutex<BTreeMap<String, JavaValue>>,
    core: ArmCore,
}

impl LgtClassDefinition {
    fn from_native(class: &LgtNativeClass, core: ArmCore) -> Self {
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
                // PUBLIC only: never block instantiation on a mis-decoded flag.
                access_flags: ClassAccessFlags::PUBLIC,
                methods,
                fields,
                statics: Mutex::new(BTreeMap::new()),
                core,
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
        self.inner.access_flags
    }

    async fn instantiate(&self, jvm: &Jvm) -> JvmResult<Box<dyn ClassInstance>> {
        let mut core = self.inner.core.clone();

        // Allocate the guest object block + a zeroed field array, link this+8 -> array.
        let alloc = (|| -> Result<u32> {
            let ptr_fields = Allocator::alloc(&mut core, FIELD_ARRAY_WORDS * 4)?;
            wie_util::ByteWrite::write_bytes(&mut core, ptr_fields, &[0u8; (FIELD_ARRAY_WORDS * 4) as usize])?;
            let ptr_raw = Allocator::alloc(&mut core, OBJ_HEADER_SIZE)?;
            write_generic(&mut core, ptr_raw, 0u32)?; // +0 (class tag, unused for now)
            write_generic(&mut core, ptr_raw + 4, 0u32)?; // +4
            write_generic(&mut core, ptr_raw + OBJ_PTR_FIELDS_OFFSET, ptr_fields)?;
            Ok(ptr_raw)
        })();
        let ptr_raw = match alloc {
            Ok(p) => p,
            Err(e) => return Err(jvm.exception("java/lang/OutOfMemoryError", &e.to_string()).await),
        };

        tracing::trace!("LGT instantiate {} -> guest {ptr_raw:#x}", self.inner.name);

        Ok(Box::new(LgtClassInstance {
            guest_ptr: ptr_raw,
            core,
            definition: self.clone(),
            jvm_fields: Arc::new(Mutex::new(BTreeMap::new())),
        }))
    }

    fn method(&self, name: &str, descriptor: &str, _is_static: bool) -> Option<Box<dyn Method>> {
        self.inner.methods.iter().find(|m| m.name == name && m.descriptor == descriptor).map(|m| {
            Box::new(LgtMethod {
                class_name: self.inner.name.clone(),
                meta: m.clone(),
                core: self.inner.core.clone(),
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
    #[allow(dead_code)] // used for guest-memory field access in checkpoint 3
    core: ArmCore,
    definition: LgtClassDefinition,
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

    // JVM-side field storage (separate from the guest field array the native code
    // uses; see module docs). Sufficient for the platform Jlet/Display glue.
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

    async fn run(&self, jvm: &Jvm, args: Box<[JavaValue]>) -> JvmResult<JavaValue> {
        // Marshal JVM args -> ARM r0..r3 (+stack). `this` (args[0] for instance
        // methods) and object args become guest pointers; primitives become raw words.
        let params: Vec<u32> = args.iter().map(marshal_arg).collect();

        tracing::debug!(
            "LGT dispatch {}.{}{} code={:#x} params={:x?}",
            self.class_name,
            self.meta.name,
            self.meta.descriptor,
            self.meta.code_ptr,
            params
        );

        let mut core = self.core.clone();
        let r0: u32 = match core.run_function(self.meta.code_ptr, &params).await {
            Ok(r) => r,
            Err(e) => {
                let msg = format!(
                    "LGT native dispatch {}.{}{} @ {:#x} failed: {e}",
                    self.class_name, self.meta.name, self.meta.descriptor, self.meta.code_ptr
                );
                return Err(jvm.exception("java/lang/Error", &msg).await);
            }
        };

        // Marshal the return value per the descriptor's return type.
        let ret = match JavaType::parse(&self.meta.descriptor) {
            JavaType::Method(_, ret) => *ret,
            _ => JavaType::Void,
        };
        Ok(marshal_return(&ret, r0))
    }
}

// ---- marshaling ----

fn marshal_arg(v: &JavaValue) -> u32 {
    match v {
        JavaValue::Void => 0,
        JavaValue::Boolean(x) => *x as u32,
        JavaValue::Byte(x) => *x as i32 as u32,
        JavaValue::Char(x) => *x as u32,
        JavaValue::Short(x) => *x as i32 as u32,
        JavaValue::Int(x) => *x as u32,
        JavaValue::Float(x) => x.to_bits(),
        JavaValue::Long(x) => *x as u32, // low word only (cp2 path has no long args)
        JavaValue::Double(x) => x.to_bits() as u32,
        JavaValue::Object(Some(inst)) => match inst.as_any().downcast_ref::<LgtClassInstance>() {
            Some(o) => o.guest_ptr,
            // Non-native object (e.g. a platform array/string): no guest backing yet.
            None => {
                tracing::warn!("LGT marshal: non-native object arg passed as null (needs cp3 platform objects)");
                0
            }
        },
        JavaValue::Object(None) => 0,
    }
}

fn marshal_return(ret: &JavaType, r0: u32) -> JavaValue {
    match ret {
        JavaType::Void => JavaValue::Void,
        JavaType::Boolean => JavaValue::Boolean(r0 != 0),
        JavaType::Byte => JavaValue::Byte(r0 as i8),
        JavaType::Char => JavaValue::Char(r0 as u16),
        JavaType::Short => JavaValue::Short(r0 as i16),
        JavaType::Int => JavaValue::Int(r0 as i32),
        JavaType::Float => JavaValue::Float(f32::from_bits(r0)),
        JavaType::Long => JavaValue::Long(r0 as i64),
        JavaType::Double => JavaValue::Double(f64::from_bits(r0 as u64)),
        // Reconstructing a JVM object from a returned guest pointer needs an
        // instance registry; not on the cp2 path (methods return void/primitive).
        JavaType::Class(_) | JavaType::Array(_) => JavaValue::Object(None),
        _ => JavaValue::Void,
    }
}

fn field_key(name: &str, descriptor: &str) -> String {
    format!("{name}:{descriptor}")
}

// ---- registration ----

/// Heuristic class-header detector over the app's `.data` segment (see
/// `docs/lgt_native_classes.md`): `+0x08` short cstring name, `+0x10` parent
/// (0 / cstring / `.data` ptr), `+0x38`/`+0x3c` 0 / `.data` count-prefixed tables.
fn scan_class_headers(core: &ArmCore, data_start: u32, data_end: u32) -> Vec<u32> {
    use wie_util::read_null_terminated_string_bytes;

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
/// ARM-backed JVM class. No-op (empty) when none are found, so the WIPI-C clet
/// path is unaffected. `system` is accepted for parity / future use.
pub async fn register_app_classes(jvm: &Jvm, core: &mut ArmCore, _system: &System, data_start: u32, data_end: u32) -> Result<Vec<String>> {
    let headers = scan_class_headers(core, data_start, data_end);
    if headers.is_empty() {
        return Ok(Vec::new());
    }

    tracing::debug!("LGT native JVM: found {} app class headers in .data", headers.len());

    // Parse all (dedupe by name), then register parents before children
    // (`register_class` resolves the superclass eagerly).
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
            let definition = LgtClassDefinition::from_native(&class, core.clone());
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
