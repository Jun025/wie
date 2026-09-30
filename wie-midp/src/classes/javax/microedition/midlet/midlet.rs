use alloc::{format, vec};

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext, get_declared_field, put_declared_field};

use crate::classes::javax::microedition::lcdui::Display;

// Not `display`: SKT titles declare a private `display` of the same type, and jvm-bytecode would give
// both one storage slot (docs/report/0382 ⚠). No game names a field this.
const DISPLAY_FIELD: &str = "wieDisplay";

// abstract class javax.microedition.midlet.MIDlet
pub struct MIDlet;

impl MIDlet {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/midlet/MIDlet",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new(
                    "getAppProperty",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    Self::get_app_property,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::FINAL,
                ),
                JavaMethodProto::new_abstract("startApp", "()V", MethodAccessFlags::PROTECTED | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new(
                    "notifyDestroyed",
                    "()V",
                    Self::notify_destroyed,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::FINAL,
                ),
            ],
            fields: vec![
                JavaFieldProto::new(
                    "currentMIDlet",
                    "Ljavax/microedition/midlet/MIDlet;",
                    FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
                ),
                JavaFieldProto::new(DISPLAY_FIELD, "Ljavax/microedition/lcdui/Display;", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.midlet.MIDlet::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        jvm.put_static_field(
            "javax/microedition/midlet/MIDlet",
            "currentMIDlet",
            "Ljavax/microedition/midlet/MIDlet;",
            this.clone(),
        )
        .await?;

        let display = jvm.new_class("javax/microedition/lcdui/Display", "()V", ()).await?;

        put_declared_field(
            jvm,
            &mut this,
            "javax/microedition/midlet/MIDlet",
            DISPLAY_FIELD,
            "Ljavax/microedition/lcdui/Display;",
            display,
        )
        .await?;

        Ok(())
    }

    async fn get_app_property(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        key: ClassInstanceRef<String>,
    ) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("javax.microedition.midlet.MIDlet::getAppProperty({this:?}, {key:?})");

        let key = JavaLangString::to_rust_string(jvm, &key).await?;
        let system_key = format!("wie.appProperty.{key}");
        let system_key = JavaLangString::from_rust_string(jvm, &system_key).await?;

        jvm.invoke_static("java/lang/System", "getProperty", "(Ljava/lang/String;)Ljava/lang/String;", (system_key,))
            .await
    }

    // The MIDlet is done: stop the way `MC_knlExit` and `System.exit` do. As a no-op stub, titles
    // that quit through here kept running and re-called it until the host stack overflowed
    // (2026-09-27 census, 10-minute runs).
    async fn notify_destroyed(_jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.midlet.MIDlet::notifyDestroyed({this:?})");

        context.system().platform().exit();

        Ok(())
    }

    pub async fn display(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Display>> {
        // A MIDlet subclass may declare its own `display` — the corpus has five (docs/report/0382).
        get_declared_field(
            jvm,
            this,
            "javax/microedition/midlet/MIDlet",
            DISPLAY_FIELD,
            "Ljavax/microedition/lcdui/Display;",
        )
        .await
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, sync::Arc};
    use core::sync::atomic::{AtomicBool, Ordering};

    use alloc::vec;

    use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
    use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
    use test_utils::{TestPlatform, TestPlatformEvent, run_jvm_test_with_system};
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_util::Result;

    use super::MIDlet;

    // Declares its own private `display`, as three SKT titles' MIDlets do (docs/report/0382).
    // jvm-bytecode keys instance storage by name, descriptor and flags, so under MIDlet's old field
    // name the two shared one slot and the game's null was MIDlet's (0262a4fe3389).
    struct TestMIDlet;

    impl TestMIDlet {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "TestMIDlet",
                parent_class: Some("javax/microedition/midlet/MIDlet"),
                interfaces: vec![],
                methods: vec![JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC)],
                fields: vec![JavaFieldProto::new(
                    "display",
                    "Ljavax/microedition/lcdui/Display;",
                    FieldAccessFlags::PRIVATE,
                )],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "javax/microedition/midlet/MIDlet", "<init>", "()V", ()).await
        }
    }

    #[test]
    fn notify_destroyed_exits() -> Result<()> {
        let exited = Arc::new(AtomicBool::new(false));
        let flag = exited.clone();
        let platform = TestPlatform::with_event_handler(move |event| {
            if matches!(event, TestPlatformEvent::Exit) {
                flag.store(true, Ordering::SeqCst);
            }
        });

        run_jvm_test_with_system(
            Box::new([crate::get_protos().into(), [TestMIDlet::as_proto()].into()]),
            Box::new(platform),
            |jvm, _system| async move {
                let midlet: ClassInstanceRef<MIDlet> = jvm.new_class("TestMIDlet", "()V", ()).await?.into();
                let _: () = jvm
                    .invoke_virtual(&midlet, "javax/microedition/midlet/MIDlet", "notifyDestroyed", "()V", ())
                    .await?;
                Ok(())
            },
        )?;

        assert!(exited.load(Ordering::SeqCst));
        Ok(())
    }

    #[test]
    fn display_is_the_midlets_own_field_not_a_subclasss() -> Result<()> {
        run_jvm_test_with_system(
            Box::new([crate::get_protos().into(), [TestMIDlet::as_proto()].into()]),
            Box::new(TestPlatform::new()),
            |jvm, _system| async move {
                let mut midlet: ClassInstanceRef<MIDlet> = jvm.new_class("TestMIDlet", "()V", ()).await?.into();
                jvm.put_field(&mut midlet, "display", "Ljavax/microedition/lcdui/Display;", None).await?;

                assert!(
                    !MIDlet::display(&jvm, &midlet).await?.is_null(),
                    "the subclass's null display did not reach MIDlet"
                );
                Ok(())
            },
        )
    }
}
