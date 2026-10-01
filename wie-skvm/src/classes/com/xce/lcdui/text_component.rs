use alloc::vec;

use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::WieJavaClassProto;

// interface com.xce.lcdui.TextComponent
//
// What a text field hands TextComponentHandler so the input method can edit it. No XCE document is in
// the repo; the method set is the one 85f03ca7389e's two text-field inner classes both implement —
// thirteen methods, identical in name and descriptor across the two. Loading either class resolves
// this interface, so without it the title loses its game thread on the first text field it builds.
pub struct TextComponent;

impl TextComponent {
    pub fn as_proto() -> WieJavaClassProto {
        const METHODS: [(&str, &str); 13] = [
            ("getCaretPosition", "()I"),
            ("getConstraints", "()I"),
            ("getMaxSize", "()I"),
            ("size", "()I"),
            ("insert", "(C)V"),
            ("delete", "()V"),
            ("clear", "()V"),
            ("replace", "(C)V"),
            ("moveCursor", "(I)V"),
            ("setCaretPosition", "(I)V"),
            ("setCaretVisible", "(Z)V"),
            ("repaint", "()V"),
            ("repaintIM", "()V"),
        ];

        WieJavaClassProto {
            name: "com/xce/lcdui/TextComponent",
            parent_class: None,
            interfaces: vec![],
            methods: METHODS
                .into_iter()
                .map(|(name, descriptor)| JavaMethodProto::new_abstract(name, descriptor, MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT))
                .collect(),
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::INTERFACE | ClassAccessFlags::ABSTRACT,
        }
    }
}
