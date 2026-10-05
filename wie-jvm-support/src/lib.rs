#![no_std]
extern crate alloc;

mod context;
mod declared_field;
pub mod guest_roots;
mod hardening;
mod jvm_implementation;
pub mod native;
mod runtime;

use alloc::{boxed::Box, format};
use rustjava_runtime::RT_RUSTJAR;

use jvm::{JavaError, Jvm, runtime::JavaLangString};
use rustjava_runtime::Runtime;

use wie_backend::System;
use wie_util::{Result, WieError};

pub use context::{WieJavaClassProto, WieJvmContext};
pub use declared_field::{get_declared_field, put_declared_field};
pub use jvm_implementation::{JvmImplementation, RustJavaJvmImplementation};
use runtime::{DefinedClasses, JvmRuntime};

pub static WIE_RUSTJAR: &str = "wie.rustjar";

/// Separator for `java.class.path`.
///
/// `ClassLoader.getSystemClassLoader` splits that property with `java.io.File.pathSeparator`,
/// which RustJava defines as `;` on Windows and `:` everywhere else. Hardcoding `:` makes the
/// whole class path parse as ONE entry on Windows: no element ends in `.rustjar`, so
/// `RustJarClassLoader` finds nothing and the first runtime class lookup unwraps a null.
/// That failure is invisible on macOS/Linux — `path_separator_matches_the_runtime` keeps the
/// two definitions in sync from any host.
const PATH_SEPARATOR: &str = path_separator(cfg!(windows));

const fn path_separator(windows: bool) -> &'static str {
    if windows { ";" } else { ":" }
}

struct SeverOnDrop(DefinedClasses);

impl Drop for SeverOnDrop {
    fn drop(&mut self) {
        self.0.sever();
    }
}

pub struct JvmSupport;

impl JvmSupport {
    pub async fn new_jvm<T>(
        system: &System,
        jar_name: Option<&str>,
        protos: Box<[Box<[WieJavaClassProto]>]>,
        properties: &[(&str, &str)],
        implementation: T,
    ) -> Result<Jvm>
    where
        T: JvmImplementation + Sync + Send + 'static,
    {
        let runtime = JvmRuntime::new(system.clone(), implementation, protos);

        let class_path = if let Some(x) = jar_name {
            format!("{RT_RUSTJAR}{PATH_SEPARATOR}{WIE_RUSTJAR}{PATH_SEPARATOR}{x}")
        } else {
            format!("{RT_RUSTJAR}{PATH_SEPARATOR}{WIE_RUSTJAR}")
        };

        let properties = [
            ("file.encoding", "EUC-KR"),
            ("java.class.path", &class_path),
            //("rustjava.disable_explicit_gc", "true"),
        ]
        .iter()
        .chain(properties.iter())
        .copied()
        .collect();
        // Owned by the JVM alone, so it drops with it — see `DefinedClasses`.
        let sever = SeverOnDrop(runtime.classes.clone());
        let jvm = Jvm::new(
            rustjava_runtime::get_bootstrap_class_loader(Box::new(runtime.clone())),
            move || {
                let _ = &sever;
                runtime.current_task_id()
            },
            properties,
        )
        .await
        .map_err(|x| WieError::FatalError(format!("Failed to create JVM: {x}")))?;

        Ok(jvm)
    }

    /// Ends a launch that ran on the startup thread `Jvm::new` attached: an error becomes a
    /// `WieError`, success detaches that thread — the event loop dispatches from now on. Left
    /// attached it counted forever: `Thread.activeCount()` read 2 in play where a handset (one
    /// system thread running `callSerially`) reads 1, and 66959afab216 starts its music only at 1.
    pub async fn finish_launch(jvm: &Jvm, result: core::result::Result<(), JavaError>) -> Result<()> {
        if let Err(err) = result {
            return Err(Self::to_wie_err(jvm, err).await);
        }
        jvm.detach_thread().map_err(|_| WieError::FatalError("detach startup thread".into()))
    }

    pub async fn to_wie_err(jvm: &Jvm, err: JavaError) -> WieError {
        let JavaError::JavaException(x) = err;
        // Formatting the trace allocates, so it fails where the heap is exhausted — an exception
        // that reached the host must still become an error there, not a panic.
        let trace = async {
            let string_writer = jvm.new_class("java/io/StringWriter", "()V", ()).await?;
            let print_writer = jvm
                .new_class("java/io/PrintWriter", "(Ljava/io/Writer;)V", (string_writer.clone(),))
                .await?;
            let _: () = jvm
                .invoke_virtual(&x, "java/lang/Throwable", "printStackTrace", "(Ljava/io/PrintWriter;)V", (print_writer,))
                .await?;
            let trace = jvm
                .invoke_virtual(&string_writer, "java/io/StringWriter", "toString", "()Ljava/lang/String;", [])
                .await?;
            JavaLangString::to_rust_string(jvm, &trace).await
        }
        .await;

        match trace {
            Ok(trace) => WieError::FatalError(format!("\n{trace}")),
            Err(error) => WieError::FatalError(format!("Java exception {x:?}; formatting its stack trace failed: {error}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, string::String};

    use jvm::{ClassInstanceRef, runtime::JavaLangString};

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{PATH_SEPARATOR, path_separator};

    /// Host-independent: the mapping itself. The runtime check below can only ever compare the
    /// *host's* separator, so on macOS/Linux it is `":" == ":"` and would not have caught the
    /// Windows-only regression this test pair exists for.
    #[test]
    fn path_separator_mapping() {
        assert_eq!(path_separator(true), ";");
        assert_eq!(path_separator(false), ":");
    }

    #[test]
    fn a_finished_launch_leaves_no_startup_thread_counted() -> Result<()> {
        run_jvm_test(Box::new([]), |jvm| async move {
            let before: i32 = jvm.invoke_static("java/lang/Thread", "activeCount", "()I", ()).await?;
            assert!(crate::JvmSupport::finish_launch(&jvm, Ok(())).await.is_ok());

            assert_eq!((before, jvm.active_thread_count()), (1, 0));

            Ok(())
        })
    }

    /// The test above locks what `finish_launch` does; this one locks that every carrier's launch
    /// ends through it. Their start functions need a whole guest to run, and only J2ME has one in
    /// `cargo test`, so this reads the source: put a launch back to `Ok(())` and it goes red.
    #[test]
    fn every_carrier_launch_ends_through_finish_launch() {
        for (carrier, source) in [
            ("skt", include_str!("../../wie-skt/src/emulator.rs")),
            ("j2me", include_str!("../../wie-j2me/src/emulator.rs")),
            ("ktf", include_str!("../../wie-ktf/src/emulator.rs")),
            ("lgt", include_str!("../../wie-lgt/src/emulator.rs")),
        ] {
            assert_eq!(source.matches("JvmSupport::finish_launch(&jvm, ").count(), 1, "{carrier}");
        }
    }

    /// `java.class.path` is split by `java.io.File.pathSeparator`, so the constant we build it
    /// with has to be the one the runtime splits with. Getting this wrong breaks class loading
    /// **only on the platform whose separator differs**, which is exactly the kind of bug a
    /// three-platform CI catches days later — assert it here so any host catches it.
    #[test]
    fn path_separator_matches_the_runtime() -> Result<()> {
        run_jvm_test(Box::new([]), |jvm| async move {
            let separator: ClassInstanceRef<String> = jvm.get_static_field("java/io/File", "pathSeparator", "Ljava/lang/String;").await?;
            let separator = JavaLangString::to_rust_string(&jvm, &separator).await?;

            assert_eq!(separator, PATH_SEPARATOR);

            Ok(())
        })
    }
}
