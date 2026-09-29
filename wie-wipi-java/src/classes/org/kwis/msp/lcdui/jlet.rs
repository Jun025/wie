use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext, get_declared_field, put_declared_field};
use wie_midp::classes::javax::microedition::midlet::MIDlet;

use crate::classes::org::kwis::msp::lcdui::{Display, EventQueue};

// class org.kwis.msp.lcdui.Jlet
pub struct Jlet;

impl Jlet {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/Jlet",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new(
                    "getActiveJlet",
                    "()Lorg/kwis/msp/lcdui/Jlet;",
                    Self::get_active_jlet,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                // KEmulator's Jlet: getCurrentJlet is public static and its body is byte-identical to getActiveJlet
                JavaMethodProto::new(
                    "getCurrentJlet",
                    "()Lorg/kwis/msp/lcdui/Jlet;",
                    Self::get_active_jlet,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getEventQueue",
                    "()Lorg/kwis/msp/lcdui/EventQueue;",
                    Self::get_event_queue,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::FINAL,
                ),
                JavaMethodProto::new(
                    "getAppProperty",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    Self::get_app_property,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::FINAL,
                ),
                JavaMethodProto::new(
                    "notifyDestroyed",
                    "()V",
                    Self::notify_destroyed,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::FINAL,
                ),
                JavaMethodProto::new_abstract(
                    "startApp",
                    "([Ljava/lang/String;)V",
                    MethodAccessFlags::PROTECTED | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new_abstract("pauseApp", "()V", MethodAccessFlags::PROTECTED | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("resumeApp", "()V", MethodAccessFlags::PROTECTED | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("destroyApp", "(Z)V", MethodAccessFlags::PROTECTED | MethodAccessFlags::ABSTRACT),
            ],
            fields: vec![
                JavaFieldProto::new("wipiMidlet", "Lnet/wie/WIPIMIDlet;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("dis", "Lorg/kwis/msp/lcdui/Display;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("eq", "Lorg/kwis/msp/lcdui/EventQueue;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new(
                    "currentJlet",
                    "Lorg/kwis/msp/lcdui/Jlet;",
                    FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
                ),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        let midlet: ClassInstanceRef<MIDlet> = jvm
            .get_static_field("javax/microedition/midlet/MIDlet", "currentMIDlet", "Ljavax/microedition/midlet/MIDlet;")
            .await?;
        jvm.put_field(&mut this, "wipiMidlet", "Lnet/wie/WIPIMIDlet;", midlet.clone()).await?;
        let _: () = jvm
            .invoke_virtual(
                &midlet,
                "net/wie/WIPIMIDlet",
                "setCurrentJlet",
                "(Lorg/kwis/msp/lcdui/Jlet;)V",
                (this.clone(),),
            )
            .await?;

        let display = jvm
            .new_class(
                "org/kwis/msp/lcdui/Display",
                "(Lorg/kwis/msp/lcdui/Jlet;Lorg/kwis/msp/lcdui/DisplayProxy;)V",
                (this.clone(), None),
            )
            .await?;

        put_declared_field(jvm, &mut this, "org/kwis/msp/lcdui/Jlet", "dis", "Lorg/kwis/msp/lcdui/Display;", display).await?;

        let event_queue = jvm
            .new_class("org/kwis/msp/lcdui/EventQueue", "(Lorg/kwis/msp/lcdui/Jlet;)V", (this.clone(),))
            .await?;

        jvm.put_field(&mut this, "eq", "Lorg/kwis/msp/lcdui/EventQueue;", event_queue).await?;

        jvm.put_static_field("org/kwis/msp/lcdui/Jlet", "currentJlet", "Lorg/kwis/msp/lcdui/Jlet;", this.clone())
            .await?;

        Ok(())
    }

    async fn get_active_jlet(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Jlet>> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::getActiveJlet");

        let jlet = jvm
            .get_static_field("org/kwis/msp/lcdui/Jlet", "currentJlet", "Lorg/kwis/msp/lcdui/Jlet;")
            .await?;

        Ok(jlet)
    }

    async fn get_event_queue(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<EventQueue>> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::getEventQueue({this:?})");

        let eq = jvm.get_field(&this, "eq", "Lorg/kwis/msp/lcdui/EventQueue;").await?;

        Ok(eq)
    }

    async fn get_app_property(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        key: ClassInstanceRef<String>,
    ) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::getAppProperty({this:?}, {key:?})");

        let midlet = jvm.get_field(&this, "wipiMidlet", "Lnet/wie/WIPIMIDlet;").await?;
        let value = jvm
            .invoke_virtual(
                &midlet,
                "net/wie/WIPIMIDlet",
                "getAppProperty",
                "(Ljava/lang/String;)Ljava/lang/String;",
                (key,),
            )
            .await?;

        Ok(value)
    }

    async fn notify_destroyed(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::notifyDestroyed({this:?})");

        // No `destroyApp` here: the platform calls that, not `notifyDestroyed`. Titles end their own
        // `destroyApp` with `notifyDestroyed()`, so calling back into it recursed until the host
        // stack overflowed (4 KTF titles in the 2026-09-27 census, at the moment they quit).
        let midlet: ClassInstanceRef<MIDlet> = jvm.get_field(&this, "wipiMidlet", "Lnet/wie/WIPIMIDlet;").await?;
        jvm.invoke_virtual(&midlet, "net/wie/WIPIMIDlet", "notifyDestroyed", "()V", ()).await
    }

    pub async fn midlet(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<MIDlet>> {
        jvm.get_field(this, "wipiMidlet", "Lnet/wie/WIPIMIDlet;").await
    }

    pub async fn display(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Display>> {
        // A Jlet subclass may declare its own `dis` — the corpus has two (docs/report/0382).
        get_declared_field(jvm, this, "org/kwis/msp/lcdui/Jlet", "dis", "Lorg/kwis/msp/lcdui/Display;").await
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc, vec};
    use core::sync::atomic::{AtomicBool, Ordering};

    use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
    use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

    use test_utils::{TestPlatform, TestPlatformEvent, run_jvm_test, run_jvm_test_with_system};
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_util::Result;

    use crate::get_protos;

    use super::Jlet;

    // A title's Jlet: counts how often the platform calls its destroyApp, and declares its own
    // `dis` as two KTF titles' Jlets do (docs/report/0382) — protected, since a private one would
    // share Jlet's slot in jvm-bytecode's storage rather than shadow it.
    struct TestJlet;

    impl TestJlet {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "TestJlet",
                parent_class: Some("org/kwis/msp/lcdui/Jlet"),
                interfaces: vec![],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("destroyApp", "(Z)V", Self::destroy_app, MethodAccessFlags::PROTECTED),
                ],
                fields: vec![
                    JavaFieldProto::new("destroyed", "I", FieldAccessFlags::STATIC),
                    JavaFieldProto::new("dis", "Lorg/kwis/msp/lcdui/Display;", FieldAccessFlags::PROTECTED),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "org/kwis/msp/lcdui/Jlet", "<init>", "()V", ()).await
        }

        async fn destroy_app(jvm: &Jvm, _: &mut WieJvmContext, _this: ClassInstanceRef<Self>, _unconditional: bool) -> JvmResult<()> {
            let count: i32 = jvm.get_static_field("TestJlet", "destroyed", "I").await?;
            jvm.put_static_field("TestJlet", "destroyed", "I", count + 1).await
        }
    }

    // notifyDestroyed ends the app and does not call back into destroyApp (which titles finish
    // with notifyDestroyed(): that loop overflowed the host stack).
    #[test]
    fn notify_destroyed_exits_without_destroy_app() -> Result<()> {
        let exited = Arc::new(AtomicBool::new(false));
        let flag = exited.clone();
        let platform = TestPlatform::with_event_handler(move |event| {
            if matches!(event, TestPlatformEvent::Exit) {
                flag.store(true, Ordering::SeqCst);
            }
        });

        run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), [TestJlet::as_proto()].into()]),
            Box::new(platform),
            |jvm, _system| async move {
                let _midlet = jvm.new_class("net/wie/WIPIMIDlet", "()V", ()).await?;
                let jlet = jvm.new_class("TestJlet", "()V", ()).await?;
                let _: () = jvm.invoke_virtual(&jlet, "org/kwis/msp/lcdui/Jlet", "notifyDestroyed", "()V", ()).await?;

                let destroyed: i32 = jvm.get_static_field("TestJlet", "destroyed", "I").await?;
                assert_eq!(destroyed, 0);
                Ok(())
            },
        )?;

        assert!(exited.load(Ordering::SeqCst));
        Ok(())
    }

    // 간호사타이쿤2 resolves getCurrentJlet as a static; it must hand back what getActiveJlet does.
    #[test]
    fn get_current_jlet_is_get_active_jlet() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let jlet = jvm.new_class("java/lang/Object", "()V", ()).await?;
            jvm.put_static_field("org/kwis/msp/lcdui/Jlet", "currentJlet", "Lorg/kwis/msp/lcdui/Jlet;", jlet.clone())
                .await?;

            let active: ClassInstanceRef<Jlet> = jvm
                .invoke_static("org/kwis/msp/lcdui/Jlet", "getActiveJlet", "()Lorg/kwis/msp/lcdui/Jlet;", ())
                .await?;
            let current: ClassInstanceRef<Jlet> = jvm
                .invoke_static("org/kwis/msp/lcdui/Jlet", "getCurrentJlet", "()Lorg/kwis/msp/lcdui/Jlet;", ())
                .await?;

            assert!(jlet.equals(&**current)?);
            assert!(active.equals(&**current)?);

            Ok(())
        })
    }

    #[test]
    fn display_is_the_jlets_own_field_not_a_subclasss() -> Result<()> {
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), [TestJlet::as_proto()].into()]),
            |jvm| async move {
                let _midlet = jvm.new_class("net/wie/WIPIMIDlet", "()V", ()).await?;
                let mut jlet: ClassInstanceRef<Jlet> = jvm.new_class("TestJlet", "()V", ()).await?.into();
                jvm.put_field(&mut jlet, "dis", "Lorg/kwis/msp/lcdui/Display;", None).await?;

                assert!(!Jlet::display(&jvm, &jlet).await?.is_null(), "the subclass's null dis did not reach Jlet");
                Ok(())
            },
        )
    }
}
