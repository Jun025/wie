use alloc::vec;

use java_constants::ClassAccessFlags;
use wie_jvm_support::WieJavaClassProto;

// interface org.kwis.msp.lwc.ActionListener
pub struct ActionListener;

impl ActionListener {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/ActionListener",
            parent_class: None,
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
            access_flags: ClassAccessFlags::INTERFACE,
        }
    }
}
