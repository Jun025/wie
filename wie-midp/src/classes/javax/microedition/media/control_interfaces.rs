use alloc::vec;

use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::WieJavaClassProto;

const ABSTRACT: MethodAccessFlags = MethodAccessFlags::PUBLIC.union(MethodAccessFlags::ABSTRACT);
const INTERFACE: ClassAccessFlags = ClassAccessFlags::PUBLIC
    .union(ClassAccessFlags::INTERFACE)
    .union(ClassAccessFlags::ABSTRACT);

// interface javax.microedition.media.control.VolumeControl
pub struct VolumeControl;

impl VolumeControl {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/media/control/VolumeControl",
            parent_class: None,
            interfaces: vec!["javax/microedition/media/Control"],
            methods: vec![
                JavaMethodProto::new_abstract("setMute", "(Z)V", ABSTRACT),
                JavaMethodProto::new_abstract("isMuted", "()Z", ABSTRACT),
                JavaMethodProto::new_abstract("setLevel", "(I)I", ABSTRACT),
                JavaMethodProto::new_abstract("getLevel", "()I", ABSTRACT),
            ],
            fields: vec![],
            access_flags: INTERFACE,
        }
    }
}

// interface javax.microedition.media.control.ToneControl
pub struct ToneControl;

impl ToneControl {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/media/control/ToneControl",
            parent_class: None,
            interfaces: vec!["javax/microedition/media/Control"],
            methods: vec![JavaMethodProto::new_abstract("setSequence", "([B)V", ABSTRACT)],
            fields: vec![],
            access_flags: INTERFACE,
        }
    }
}
