use alloc::{collections::BTreeMap, format, string::String, vec::Vec};
use core::fmt::{self, Debug, Formatter};

use jvm::{Field, JavaType, JavaValue};
use jvm_types::FieldAccessFlags;
use spin::Mutex;
use wipi_types::lgt::java::LgtJavaClassField as RawJavaField;

use wie_core_arm::{Allocator, ArmCore};
use wie_util::{Result, read_generic, read_null_terminated_string_bytes, write_generic, write_null_terminated_string_bytes};

#[derive(Clone)]
pub struct JavaField {
    pub ptr_raw: u32,
    core: ArmCore,
}

impl JavaField {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore) -> Self {
        Self { ptr_raw, core: core.clone() }
    }

    pub fn new(
        core: &mut ArmCore,
        ptr_raw: u32,
        ptr_class: u32,
        name: &str,
        descriptor: &str,
        access_flags: FieldAccessFlags,
        word_index: u32,
    ) -> Result<Self> {
        let ptr_name = Allocator::alloc(core, (name.len() + 1) as u32)?;
        write_null_terminated_string_bytes(core, ptr_name, name.as_bytes())?;

        let ptr_descriptor = Allocator::alloc(core, (descriptor.len() + 1) as u32)?;
        write_null_terminated_string_bytes(core, ptr_descriptor, descriptor.as_bytes())?;

        write_generic(
            core,
            ptr_raw,
            RawJavaField {
                ptr_class,
                ptr_name,
                ptr_descriptor,
                flags: access_flags.bits(),
                unk2: 0,
                word_index,
            },
        )?;

        Ok(Self::from_raw(ptr_raw, core))
    }

    pub fn raw(&self) -> Result<RawJavaField> {
        read_generic(&self.core, self.ptr_raw)
    }

    pub fn word_index(&self) -> Result<u32> {
        Ok(self.raw()?.word_index)
    }
}

impl Field for JavaField {
    fn name(&self) -> String {
        String::from_utf8(read_null_terminated_string_bytes(&self.core, self.raw().unwrap().ptr_name).unwrap()).unwrap()
    }

    fn descriptor(&self) -> String {
        String::from_utf8(read_null_terminated_string_bytes(&self.core, self.raw().unwrap().ptr_descriptor).unwrap()).unwrap()
    }

    fn access_flags(&self) -> FieldAccessFlags {
        FieldAccessFlags::from_bits_truncate(self.raw().unwrap().flags as _)
    }
}

impl Debug for JavaField {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("JavaField").field("ptr_raw", &self.ptr_raw).finish()
    }
}

#[derive(Debug)]
pub struct JavaReferenceField {
    pub word_index: u32,
}

impl Field for JavaReferenceField {
    fn name(&self) -> String {
        format!("<reference-word-{}>", self.word_index)
    }

    fn descriptor(&self) -> String {
        "Ljava/lang/Object;".into()
    }

    fn access_flags(&self) -> FieldAccessFlags {
        FieldAccessFlags::empty()
    }
}

#[derive(Debug)]
pub struct JavaStaticReferenceField {
    pub ptr_class: u32,
    pub word_index: u32,
}

impl Field for JavaStaticReferenceField {
    fn name(&self) -> String {
        format!("<static-reference-word-{}>", self.word_index)
    }

    fn descriptor(&self) -> String {
        "Ljava/lang/Object;".into()
    }

    fn access_flags(&self) -> FieldAccessFlags {
        FieldAccessFlags::STATIC
    }
}

/// A host-declared instance field stored outside instance storage (see `JavaClassAbi::host_field`).
#[derive(Debug)]
pub struct JavaHostField {
    pub name: String,
    pub descriptor: String,
}

impl Field for JavaHostField {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn descriptor(&self) -> String {
        self.descriptor.clone()
    }

    fn access_flags(&self) -> FieldAccessFlags {
        FieldAccessFlags::PRIVATE
    }
}

// Keyed by (core id, instance pointer): two emulators in one process hand out the same guest
// addresses. ponytail: one global lock, taken only by host-field access and by instance
// create/destroy while the table is non-empty.
static HOST_FIELD_VALUES: Mutex<BTreeMap<(usize, u32), BTreeMap<String, JavaValue>>> = Mutex::new(BTreeMap::new());

impl JavaHostField {
    pub fn get(&self, core: &ArmCore, ptr_instance: u32) -> JavaValue {
        HOST_FIELD_VALUES
            .lock()
            .get(&(core.id(), ptr_instance))
            .and_then(|fields| fields.get(&self.name).cloned())
            .unwrap_or_else(|| JavaType::parse(&self.descriptor).default())
    }

    pub fn put(&self, core: &ArmCore, ptr_instance: u32, value: JavaValue) {
        HOST_FIELD_VALUES
            .lock()
            .entry((core.id(), ptr_instance))
            .or_default()
            .insert(self.name.clone(), value);
    }

    /// Drops every value a core's instances held — the values reference that core's objects, so
    /// leaving them would keep the core alive and hand its entries to a later core at its address.
    pub fn forget_core(core: &ArmCore) {
        let id = core.id();
        let values = HOST_FIELD_VALUES
            .lock()
            .extract_if(.., |(core_id, _), _| *core_id == id)
            .collect::<Vec<_>>();
        drop(values);
    }

    /// Drops whatever an earlier instance at this address left behind.
    pub fn forget_instance(core: &ArmCore, ptr_instance: u32) {
        let mut values = HOST_FIELD_VALUES.lock();
        if !values.is_empty() {
            values.remove(&(core.id(), ptr_instance));
        }
    }
}
