use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{
    javax::microedition::lcdui::{ChoiceGroup, Command, Font, Image},
    net::wie::{KeyboardEventType, MIDPKeyCode},
};

// class javax.microedition.lcdui.List
//
// A Form holding one label-less ChoiceGroup that every Choice method forwards to, so the list gets
// the Form's layout, scrolling and focus for free. IMPLICIT is an EXCLUSIVE group whose FIRE also
// sends the select command (List.SELECT_COMMAND unless replaced) to the CommandListener.
// ponytail: `List instanceof Form` is true here; nothing a game can observe depends on it short of
// that test. Make List a Screen of its own if a title ever branches on it.
pub struct List;

const CHOICE: &str = "javax/microedition/lcdui/ChoiceGroup";
const CHOICE_DESC: &str = "Ljavax/microedition/lcdui/ChoiceGroup;";
const COMMAND_DESC: &str = "Ljavax/microedition/lcdui/Command;";
const IMPLICIT: i32 = 3;
const EXCLUSIVE: i32 = 1;

macro_rules! forward {
    ($fn:ident, $name:literal, $desc:literal, ($($arg:ident: $ty:ty),*) -> $ret:ty) => {
        async fn $fn(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, $($arg: $ty),*) -> JvmResult<$ret> {
            let choice: ClassInstanceRef<ChoiceGroup> = jvm.get_field(&this, "choice", CHOICE_DESC).await?;
            jvm.invoke_virtual(&choice, CHOICE, $name, $desc, ($($arg,)*)).await
        }
    };
}

impl List {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/List",
            parent_class: Some("javax/microedition/lcdui/Form"),
            interfaces: vec!["javax/microedition/lcdui/Choice"],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;I[Ljava/lang/String;[Ljavax/microedition/lcdui/Image;)V",
                    Self::init_with_elements,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("size", "()I", Self::size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getString", "(I)Ljava/lang/String;", Self::get_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getImage",
                    "(I)Ljavax/microedition/lcdui/Image;",
                    Self::get_image,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "append",
                    "(Ljava/lang/String;Ljavax/microedition/lcdui/Image;)I",
                    Self::append,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "insert",
                    "(ILjava/lang/String;Ljavax/microedition/lcdui/Image;)V",
                    Self::insert,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("delete", "(I)V", Self::delete, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("deleteAll", "()V", Self::delete_all, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "set",
                    "(ILjava/lang/String;Ljavax/microedition/lcdui/Image;)V",
                    Self::set,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("isSelected", "(I)Z", Self::is_selected, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getSelectedIndex", "()I", Self::get_selected_index, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getSelectedFlags", "([Z)I", Self::get_selected_flags, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setSelectedIndex", "(IZ)V", Self::set_selected_index, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setSelectedFlags", "([Z)V", Self::set_selected_flags, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setFitPolicy", "(I)V", Self::set_fit_policy, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getFitPolicy", "()I", Self::get_fit_policy, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setFont",
                    "(ILjavax/microedition/lcdui/Font;)V",
                    Self::set_font,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getFont", "(I)Ljavax/microedition/lcdui/Font;", Self::get_font, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setSelectCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    Self::set_select_command,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("handleKeyEvent", "(II)V", Self::handle_key_event, MethodAccessFlags::empty()),
            ],
            fields: vec![
                JavaFieldProto::new(
                    "SELECT_COMMAND",
                    COMMAND_DESC,
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new("choice", CHOICE_DESC, FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("listType", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("selectCommand", COMMAND_DESC, FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        let label = JavaLangString::from_rust_string(jvm, "").await?;
        let command = jvm
            .new_class("javax/microedition/lcdui/Command", "(Ljava/lang/String;II)V", (label, 1, 0)) // SCREEN, priority 0
            .await?;

        jvm.put_static_field("javax/microedition/lcdui/List", "SELECT_COMMAND", COMMAND_DESC, command)
            .await
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, title: ClassInstanceRef<String>, list_type: i32) -> JvmResult<()> {
        let elements = jvm.instantiate_array("Ljava/lang/String;", 0).await?;
        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/List",
            "<init>",
            "(Ljava/lang/String;I[Ljava/lang/String;[Ljavax/microedition/lcdui/Image;)V",
            (title, list_type, elements, None),
        )
        .await
    }

    async fn init_with_elements(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        title: ClassInstanceRef<String>,
        list_type: i32,
        elements: ClassInstanceRef<Array<ClassInstanceRef<String>>>,
        images: ClassInstanceRef<Array<ClassInstanceRef<Image>>>,
    ) -> JvmResult<()> {
        if !(1..=3).contains(&list_type) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid list type").await);
        }
        let _: () = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Form", "<init>", "(Ljava/lang/String;)V", (title,))
            .await?;

        let group_type = if list_type == IMPLICIT { EXCLUSIVE } else { list_type };
        let choice = jvm
            .new_class(
                CHOICE,
                "(Ljava/lang/String;I[Ljava/lang/String;[Ljavax/microedition/lcdui/Image;)V",
                (None, group_type, elements, images),
            )
            .await?;
        let _: i32 = jvm
            .invoke_virtual(
                &this,
                "javax/microedition/lcdui/Form",
                "append",
                "(Ljavax/microedition/lcdui/Item;)I",
                (choice.clone(),),
            )
            .await?;
        jvm.put_field(&mut this, "choice", CHOICE_DESC, choice).await?;
        jvm.put_field(&mut this, "listType", "I", list_type).await?;
        let select: ClassInstanceRef<Command> = jvm
            .get_static_field("javax/microedition/lcdui/List", "SELECT_COMMAND", COMMAND_DESC)
            .await?;
        jvm.put_field(&mut this, "selectCommand", COMMAND_DESC, select).await
    }

    forward!(size, "size", "()I", () -> i32);
    forward!(get_string, "getString", "(I)Ljava/lang/String;", (index: i32) -> ClassInstanceRef<String>);
    forward!(get_image, "getImage", "(I)Ljavax/microedition/lcdui/Image;", (index: i32) -> ClassInstanceRef<Image>);
    forward!(append, "append", "(Ljava/lang/String;Ljavax/microedition/lcdui/Image;)I", (text: ClassInstanceRef<String>, image: ClassInstanceRef<Image>) -> i32);
    forward!(insert, "insert", "(ILjava/lang/String;Ljavax/microedition/lcdui/Image;)V", (index: i32, text: ClassInstanceRef<String>, image: ClassInstanceRef<Image>) -> ());
    forward!(delete, "delete", "(I)V", (index: i32) -> ());
    forward!(delete_all, "deleteAll", "()V", () -> ());
    forward!(set, "set", "(ILjava/lang/String;Ljavax/microedition/lcdui/Image;)V", (index: i32, text: ClassInstanceRef<String>, image: ClassInstanceRef<Image>) -> ());
    forward!(is_selected, "isSelected", "(I)Z", (index: i32) -> bool);
    forward!(get_selected_index, "getSelectedIndex", "()I", () -> i32);
    forward!(get_selected_flags, "getSelectedFlags", "([Z)I", (flags: ClassInstanceRef<Array<bool>>) -> i32);
    forward!(set_selected_index, "setSelectedIndex", "(IZ)V", (index: i32, selected: bool) -> ());
    forward!(set_selected_flags, "setSelectedFlags", "([Z)V", (flags: ClassInstanceRef<Array<bool>>) -> ());
    forward!(set_fit_policy, "setFitPolicy", "(I)V", (policy: i32) -> ());
    forward!(get_fit_policy, "getFitPolicy", "()I", () -> i32);
    forward!(set_font, "setFont", "(ILjavax/microedition/lcdui/Font;)V", (index: i32, font: ClassInstanceRef<Font>) -> ());
    forward!(get_font, "getFont", "(I)Ljavax/microedition/lcdui/Font;", (index: i32) -> ClassInstanceRef<Font>);

    async fn set_select_command(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        command: ClassInstanceRef<Command>,
    ) -> JvmResult<()> {
        jvm.put_field(&mut this, "selectCommand", COMMAND_DESC, command).await
    }

    async fn handle_key_event(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, event_type: i32, code: i32) -> JvmResult<()> {
        let _: () = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Form", "handleKeyEvent", "(II)V", (event_type, code))
            .await?;

        let list_type: i32 = jvm.get_field(&this, "listType", "I").await?;
        let fire = code == MIDPKeyCode::FIRE as i32 || code == MIDPKeyCode::KEY_NUM5 as i32;
        if list_type != IMPLICIT || !fire || event_type != KeyboardEventType::KeyPressed as i32 {
            return Ok(());
        }
        let command: ClassInstanceRef<Command> = jvm.get_field(&this, "selectCommand", COMMAND_DESC).await?;
        if command.is_null() {
            return Ok(());
        }
        let _: bool = jvm
            .invoke_virtual(
                &this,
                "javax/microedition/lcdui/Displayable",
                "dispatchCommand",
                "(Ljavax/microedition/lcdui/Command;)Z",
                (command,),
            )
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::runtime::JavaLangString;
    use test_utils::run_jvm_test;

    use crate::{
        classes::net::wie::{KeyboardEventType, MIDPKeyCode},
        get_protos,
    };

    // An IMPLICIT list: DOWN moves the focus, FIRE selects the focused element.
    #[test]
    fn implicit_list_selects_the_focused_element() {
        let result = run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let title = JavaLangString::from_rust_string(&jvm, "Menu").await?;
            let list = jvm
                .new_class("javax/microedition/lcdui/List", "(Ljava/lang/String;I)V", (title, 3))
                .await?;
            for label in ["Play", "Quit"] {
                let label = JavaLangString::from_rust_string(&jvm, label).await?;
                let _: i32 = jvm
                    .invoke_virtual(
                        &list,
                        "javax/microedition/lcdui/List",
                        "append",
                        "(Ljava/lang/String;Ljavax/microedition/lcdui/Image;)I",
                        (label, None),
                    )
                    .await?;
            }
            for key in [MIDPKeyCode::DOWN, MIDPKeyCode::FIRE] {
                let _: () = jvm
                    .invoke_virtual(
                        &list,
                        "javax/microedition/lcdui/List",
                        "handleKeyEvent",
                        "(II)V",
                        (KeyboardEventType::KeyPressed as i32, key as i32),
                    )
                    .await?;
            }
            let size: i32 = jvm.invoke_virtual(&list, "javax/microedition/lcdui/List", "size", "()I", ()).await?;
            let selected: i32 = jvm
                .invoke_virtual(&list, "javax/microedition/lcdui/List", "getSelectedIndex", "()I", ())
                .await?;
            assert_eq!((size, selected), (2, 1));
            Ok(())
        });
        assert!(result.is_ok(), "{result:?}");
    }
}
