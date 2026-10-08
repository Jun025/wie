use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use super::{GMenubarForm, GTextListener};

// class com.ktf.kfc.GTextField -- a text field placed on a GMenubarForm. f07cbc782828 makes one in its
// startApp with `<init>(Lcom/ktf/kfc/GMenubarForm;Ljava/lang/String;I)V`, then calls the inherited
// `setMaxLength(I)V` and `getGTextListener()` (once per field, so it is the field's own object).
// Text editing is TextComponent's; the form is not told about the field.
// ponytail: `constraint` is not interpreted — nothing observed depends on it.
pub struct GTextField;

impl GTextField {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/ktf/kfc/GTextField",
            parent_class: Some("org/kwis/msp/lwc/TextComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Lcom/ktf/kfc/GMenubarForm;Ljava/lang/String;I)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "getGTextListener",
                    "()Lcom/ktf/kfc/GTextListener;",
                    Self::get_g_text_listener,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            fields: vec![JavaFieldProto::new("listener", "Lcom/ktf/kfc/GTextListener;", FieldAccessFlags::PRIVATE)],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        form: ClassInstanceRef<GMenubarForm>,
        text: ClassInstanceRef<String>,
        constraint: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GTextField::<init>({this:?}, {form:?}, {text:?}, {constraint})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/TextComponent", "<init>", "()V", ()).await?;
        if !text.is_null() {
            let _: () = jvm
                .invoke_virtual(&this, "org/kwis/msp/lwc/TextComponent", "setString", "(Ljava/lang/String;)V", (text,))
                .await?;
        }
        Ok(())
    }

    async fn get_g_text_listener(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<GTextListener>> {
        tracing::debug!("com.ktf.kfc.GTextField::getGTextListener({this:?})");

        let listener: ClassInstanceRef<GTextListener> = jvm.get_field(&this, "listener", "Lcom/ktf/kfc/GTextListener;").await?;
        if !listener.is_null() {
            return Ok(listener);
        }
        let listener: ClassInstanceRef<GTextListener> = jvm.new_class("com/ktf/kfc/GTextListener", "()V", ()).await?.into();
        let mut this = this;
        jvm.put_field(&mut this, "listener", "Lcom/ktf/kfc/GTextListener;", listener.clone())
            .await?;
        Ok(listener)
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, runtime::JavaLangString};
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    // f07cbc782828 builds its fields in startApp and configures the listener of each; a missing
    // constructor or getter ends its boot with «Method … not found».
    #[test]
    fn g_text_field_keeps_its_text_and_one_listener() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let form = jvm.new_class("com/ktf/kfc/GForm", "(IIII)V", (0, 0, 10, 10)).await?;
            let text = JavaLangString::from_rust_string(&jvm, "ab").await?;
            let field = jvm
                .new_class(
                    "com/ktf/kfc/GTextField",
                    "(Lcom/ktf/kfc/GMenubarForm;Ljava/lang/String;I)V",
                    (form, text, 0),
                )
                .await?;
            let read = jvm
                .invoke_virtual(&field, "org/kwis/msp/lwc/TextComponent", "getString", "()Ljava/lang/String;", ())
                .await?;
            assert_eq!(JavaLangString::to_rust_string(&jvm, &read).await?, "ab");

            let get = |jvm: &jvm::Jvm, field| {
                let jvm = jvm.clone();
                async move {
                    let listener: ClassInstanceRef<()> = jvm
                        .invoke_virtual(&field, "com/ktf/kfc/GTextField", "getGTextListener", "()Lcom/ktf/kfc/GTextListener;", ())
                        .await?;
                    Ok::<_, jvm::JavaError>(listener)
                }
            };
            let first = get(&jvm, field.clone()).await?;
            let second = get(&jvm, field.clone()).await?;
            assert!(jvm.is_instance(&**first, "com/ktf/kfc/GTextListener"));
            assert_eq!(first.identity(), second.identity(), "one listener per field");
            let modes = jvm.instantiate_array("I", 1).await?;
            let _: () = jvm
                .invoke_virtual(&first, "com/ktf/kfc/GTextListener", "setIMEModes", "([I)V", (modes,))
                .await?;

            Ok(())
        })
    }
}
