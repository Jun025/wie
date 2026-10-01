use alloc::{vec, vec::Vec};

use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// MIDP 2.0 constraint values.
const CONSTANTS: [(&str, i32); 13] = [
    ("ANY", 0),
    ("EMAILADDR", 1),
    ("NUMERIC", 2),
    ("PHONENUMBER", 3),
    ("URL", 4),
    ("DECIMAL", 5),
    ("PASSWORD", 0x10000),
    ("UNEDITABLE", 0x20000),
    ("SENSITIVE", 0x40000),
    ("NON_PREDICTIVE", 0x80000),
    ("INITIAL_CAPS_WORD", 0x100000),
    ("INITIAL_CAPS_SENTENCE", 0x200000),
    ("CONSTRAINT_MASK", 0xFFFF),
];

// class javax.microedition.lcdui.TextField
//
// Constants only. The measured caller (85f03ca7389e's own text-field classes) reads CONSTRAINT_MASK and
// PASSWORD and never constructs one, so there is no <init> — a title that does gets NoSuchMethodError,
// which names the next thing to add.
pub struct TextField;

impl TextField {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/TextField",
            parent_class: Some("javax/microedition/lcdui/Item"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC)],
            fields: CONSTANTS
                .iter()
                .map(|(name, _)| JavaFieldProto::new(name, "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL))
                .collect::<Vec<_>>(),
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        for (name, value) in CONSTANTS {
            jvm.put_static_field("javax/microedition/lcdui/TextField", name, "I", value).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use test_utils::run_jvm_test;

    use crate::get_protos;

    #[test]
    fn text_field_constants_match_midp() {
        let result = run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let mask: i32 = jvm.get_static_field("javax/microedition/lcdui/TextField", "CONSTRAINT_MASK", "I").await?;
            let password: i32 = jvm.get_static_field("javax/microedition/lcdui/TextField", "PASSWORD", "I").await?;
            let numeric: i32 = jvm.get_static_field("javax/microedition/lcdui/TextField", "NUMERIC", "I").await?;
            assert_eq!((mask, password, numeric), (0xFFFF, 0x10000, 2));
            Ok(())
        });
        assert!(result.is_ok(), "{result:?}");
    }
}
