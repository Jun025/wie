use alloc::{string::String, vec::Vec};

use serde::Deserialize;

const LGT_JAVA_ABI_TOML: &str = include_str!("../../../data/lgt_java_abi.toml");

lazy_static::lazy_static! {
    pub static ref JAVA_ABI: JavaAbi = toml::from_str(LGT_JAVA_ABI_TOML).expect("parse data/lgt_java_abi.toml");
}

pub const CLASS_NATIVE_NAME_FIELD: &str = "nativeClassName";
pub const CLASS_INITIALIZATION_STATE_FIELD: &str = "initializationState";
pub const WORD_FIELD_DESCRIPTOR: &str = "I";

#[derive(Deserialize)]
pub struct JavaAbi {
    #[serde(default)]
    pub class: Vec<JavaClassAbi>,
}

#[derive(Deserialize)]
pub struct JavaClassAbi {
    pub name: String,
    pub vtable_size: Option<usize>,
    #[serde(default)]
    pub vtable: Vec<JavaVtableIndex>,
    #[serde(default)]
    pub field: Vec<JavaFieldIndex>,
    /// Host-declared instance fields the phone's layout has no words for. They are kept outside
    /// instance storage (`field::JavaHostField`), so they cannot overwrite a guest subclass's own
    /// fields. Primitive descriptors only — the collector never sees these values.
    #[serde(default)]
    pub host_field: Vec<JavaHostFieldAbi>,
}

#[derive(Deserialize)]
pub struct JavaHostFieldAbi {
    pub name: String,
    pub descriptor: String,
}

#[derive(Deserialize)]
pub struct JavaVtableIndex {
    pub name: String,
    pub descriptor: String,
    pub index: usize,
}

#[derive(Deserialize)]
pub struct JavaFieldIndex {
    pub name: String,
    pub descriptor: String,
    pub index: u32,
}

impl JavaAbi {
    pub fn class(&self, class_name: &str) -> Option<&JavaClassAbi> {
        self.class.iter().find(|class| class.name == class_name)
    }

    /// Cheap pre-check on the field-lookup miss path: only a name listed somewhere as a host field
    /// is worth reading the class name for.
    pub fn is_host_field_name(&self, name: &str) -> bool {
        self.class.iter().any(|class| class.host_field.iter().any(|field| field.name == name))
    }
}

impl JavaClassAbi {
    pub fn is_host_field(&self, name: &str, descriptor: &str) -> bool {
        self.host_field.iter().any(|field| field.name == name && field.descriptor == descriptor)
    }

    pub fn vtable_index(&self, method_name: &str, descriptor: &str) -> Option<usize> {
        self.vtable
            .iter()
            .find(|entry| entry.name == method_name && entry.descriptor == descriptor)
            .map(|entry| entry.index)
    }
}
