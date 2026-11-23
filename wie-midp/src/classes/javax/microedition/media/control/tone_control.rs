use alloc::vec;

use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::WieJavaClassProto;

// interface javax.microedition.media.control.ToneControl
pub struct ToneControl;

impl ToneControl {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/media/control/ToneControl",
            parent_class: None,
            interfaces: vec!["javax/microedition/media/Control"],
            methods: vec![JavaMethodProto::new_abstract(
                "setSequence",
                "([B)V",
                MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
            )],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::INTERFACE | ClassAccessFlags::ABSTRACT,
        }
    }
}
