use alloc::vec;

use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use wie_jvm_support::WieJavaClassProto;

// interface org.kwis.msp.lwc.GrabKeyListener — `grabKeyNotify(type, key, param)`, the signature
// 85e94babc247's client.bin carries.
pub struct GrabKeyListener;

impl GrabKeyListener {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/GrabKeyListener",
            parent_class: None,
            interfaces: vec![],
            methods: vec![JavaMethodProto::new_abstract(
                "grabKeyNotify",
                "(IILjava/lang/Object;)Z",
                MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
            )],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::INTERFACE | ClassAccessFlags::ABSTRACT,
        }
    }
}
