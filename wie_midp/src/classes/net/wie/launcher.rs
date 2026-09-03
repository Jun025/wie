use alloc::{boxed::Box, vec};

use java_class_proto::{JavaMethodProto, MethodBody};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::{Class, ClassLoader, String};
use jvm::{ClassInstanceRef, JavaError, JavaValue, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::midlet::MIDlet;

// class net.wie.Launcher
pub struct Launcher;

impl Launcher {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/Launcher",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("start", "(Ljava/lang/String;)V", Self::start, MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "startMIDlet",
                    "(Ljavax/microedition/midlet/MIDlet;)V",
                    Self::start_midlet,
                    MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn start(jvm: &Jvm, _context: &mut WieJvmContext, main_class: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("net.wie.Launcher::start({main_class:?})");

        // Load the MIDlet through the SYSTEM class loader, not through an ambient
        // `jvm.new_class`.
        //
        // `net/wie/Launcher` is itself defined by `RustJarClassLoader` (net.wie.rustjar is
        // a `.rustjar` class-path entry). `jvm.new_class` resolves against the *calling*
        // class's loader, and that loader's parent is null while `findClass` skips every
        // entry that does not end in `.rustjar` — so from here the guest jar is
        // unreachable and any MIDlet name comes back `NoClassDefFoundError`. The system
        // loader is the `URLClassLoader` that owns the jar (its parent is the rustjar
        // loader, so runtime classes still resolve), which is also where the JLS says an
        // application class belongs.
        let class_loader: ClassInstanceRef<ClassLoader> = jvm
            .invoke_static("java/lang/ClassLoader", "getSystemClassLoader", "()Ljava/lang/ClassLoader;", ())
            .await?;

        let main_class: ClassInstanceRef<Class> = jvm
            .invoke_virtual(&class_loader, "loadClass", "(Ljava/lang/String;)Ljava/lang/Class;", (main_class,))
            .await?;

        let main_class: ClassInstanceRef<MIDlet> = jvm.invoke_virtual(&main_class, "newInstance", "()Ljava/lang/Object;", ()).await?;

        jvm.invoke_static("net/wie/Launcher", "startMIDlet", "(Ljavax/microedition/midlet/MIDlet;)V", (main_class,))
            .await
    }

    async fn start_midlet(jvm: &Jvm, context: &mut WieJvmContext, midlet: ClassInstanceRef<MIDlet>) -> JvmResult<()> {
        tracing::debug!("net.wie.Launcher::startMIDlet({midlet:?})");

        // run startApp
        let _: () = jvm.invoke_virtual(&midlet, "startApp", "()V", (None,)).await?;

        // spawn event loop
        context.spawn(jvm, Box::new(EventLoopRunner))?;

        Ok(())
    }
}

struct EventLoopRunner;

#[async_trait::async_trait]
impl MethodBody<JavaError, WieJvmContext> for EventLoopRunner {
    async fn call(&self, jvm: &Jvm, _context: &mut WieJvmContext, _args: Box<[JavaValue]>) -> Result<JavaValue, JavaError> {
        // +33 pin: `attach_thread` is async and takes the Java `Thread` object.
        // This is the host-side event loop, which has no Java Thread — `None`,
        // the same value RustJava's own bootstrap passes (jvm/src/jvm.rs:87).
        jvm.attach_thread(None).await?;

        // event loop
        let event_queue = jvm
            .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
            .await?;

        let event = jvm.instantiate_array("I", 4).await?;
        loop {
            let _: () = jvm.invoke_virtual(&event_queue, "getNextEvent", "([I)V", (event.clone(),)).await?;
            let _: () = jvm.invoke_virtual(&event_queue, "dispatchEvent", "([I)V", (event.clone(),)).await?;
        }
    }
}
