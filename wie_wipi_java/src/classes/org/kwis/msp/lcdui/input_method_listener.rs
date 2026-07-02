use alloc::vec;

use java_constants::ClassAccessFlags;
use wie_jvm_support::WieJavaClassProto;

// interface org.kwis.msp.lcdui.InputMethodListener
// Registered so KTF init-time load_java_class resolves it; games implement it themselves.
pub struct InputMethodListener;

impl InputMethodListener {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/InputMethodListener",
            parent_class: None,
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
            access_flags: ClassAccessFlags::INTERFACE,
        }
    }
}
