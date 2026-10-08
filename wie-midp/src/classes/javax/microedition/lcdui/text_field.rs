use alloc::{vec, vec::Vec};

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

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
// The constants (85f03ca7389e's own text-field classes read CONSTRAINT_MASK and PASSWORD), and since
// 2026-10-08 a text field a title can construct and read back: minitruco builds two in its options
// Form at MIDlet construction and died on NoSuchMethodError before its first paint.
// ponytail: it is a StringItem underneath — shown with its label, read and written by the game, but
// not editable from the keypad. Add key entry (and the TextBox editor) if a title needs the player to type.
pub struct TextField;

impl TextField {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/TextField",
            parent_class: Some("javax/microedition/lcdui/StringItem"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Ljava/lang/String;II)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getString", "()Ljava/lang/String;", Self::get_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setString", "(Ljava/lang/String;)V", Self::set_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("size", "()I", Self::size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getMaxSize", "()I", Self::get_max_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setMaxSize", "(I)I", Self::set_max_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getConstraints", "()I", Self::get_constraints, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setConstraints", "(I)V", Self::set_constraints, MethodAccessFlags::PUBLIC),
            ],
            fields: CONSTANTS
                .iter()
                .map(|(name, _)| JavaFieldProto::new(name, "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL))
                .chain([
                    JavaFieldProto::new("maxSize", "I", FieldAccessFlags::PRIVATE),
                    JavaFieldProto::new("constraints", "I", FieldAccessFlags::PRIVATE),
                ])
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

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        text: ClassInstanceRef<String>,
        max_size: i32,
        constraints: i32,
    ) -> JvmResult<()> {
        if max_size <= 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "maxSize must be positive").await);
        }
        let _: () = jvm
            .invoke_special(
                &this,
                "javax/microedition/lcdui/StringItem",
                "<init>",
                "(Ljava/lang/String;Ljava/lang/String;)V",
                (label, text),
            )
            .await?;
        jvm.put_field(&mut this, "maxSize", "I", max_size).await?;
        jvm.put_field(&mut this, "constraints", "I", constraints).await
    }

    async fn get_string(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        let text: ClassInstanceRef<String> = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/StringItem", "getText", "()Ljava/lang/String;", ())
            .await?;
        if text.is_null() {
            return Ok(JavaLangString::from_rust_string(jvm, "").await?.into());
        }

        Ok(text)
    }

    async fn set_string(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, text: ClassInstanceRef<String>) -> JvmResult<()> {
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/StringItem", "setText", "(Ljava/lang/String;)V", (text,))
            .await
    }

    async fn size(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let text = Self::get_string(jvm, context, this).await?;

        jvm.invoke_virtual(&text, "java/lang/String", "length", "()I", ()).await
    }

    async fn get_max_size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "maxSize", "I").await
    }

    async fn set_max_size(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, max_size: i32) -> JvmResult<i32> {
        if max_size <= 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "maxSize must be positive").await);
        }
        jvm.put_field(&mut this, "maxSize", "I", max_size).await?;

        Ok(max_size)
    }

    async fn get_constraints(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "constraints", "I").await
    }

    async fn set_constraints(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, constraints: i32) -> JvmResult<()> {
        jvm.put_field(&mut this, "constraints", "I", constraints).await
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::runtime::JavaLangString;
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

    #[test]
    fn text_field_holds_what_the_game_writes() {
        let result = run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let label = JavaLangString::from_rust_string(&jvm, "Margin").await?;
            let text = JavaLangString::from_rust_string(&jvm, "12").await?;
            let field = jvm
                .new_class(
                    "javax/microedition/lcdui/TextField",
                    "(Ljava/lang/String;Ljava/lang/String;II)V",
                    (label, text, 3, 2),
                )
                .await?;
            let size: i32 = jvm
                .invoke_virtual(&field, "javax/microedition/lcdui/TextField", "size", "()I", ())
                .await?;
            let constraints: i32 = jvm
                .invoke_virtual(&field, "javax/microedition/lcdui/TextField", "getConstraints", "()I", ())
                .await?;
            assert_eq!((size, constraints), (2, 2));

            let _: () = jvm
                .invoke_virtual(
                    &field,
                    "javax/microedition/lcdui/TextField",
                    "setString",
                    "(Ljava/lang/String;)V",
                    (None,),
                )
                .await?;
            let text = jvm
                .invoke_virtual(&field, "javax/microedition/lcdui/TextField", "getString", "()Ljava/lang/String;", ())
                .await?;
            assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, ""); // null reads back as empty
            Ok(())
        });
        assert!(result.is_ok(), "{result:?}");
    }
}
