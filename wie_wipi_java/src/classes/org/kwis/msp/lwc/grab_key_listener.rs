use alloc::vec;

use java_constants::ClassAccessFlags;
use wie_jvm_support::WieJavaClassProto;

// interface org.kwis.msp.lwc.GrabKeyListener
pub struct GrabKeyListener;

impl GrabKeyListener {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/GrabKeyListener",
            parent_class: None,
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
            access_flags: ClassAccessFlags::INTERFACE,
        }
    }
}
