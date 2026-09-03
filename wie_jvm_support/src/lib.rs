#![no_std]
extern crate alloc;

mod context;
mod hardening;
mod jvm_implementation;
mod runtime;

use alloc::{boxed::Box, format};

use java_runtime::{RT_RUSTJAR, Runtime};
use jvm::{JavaError, Jvm, runtime::JavaLangString};

use wie_backend::System;
use wie_util::{Result, WieError};

pub use context::{WieJavaClassProto, WieJvmContext};
pub use jvm_implementation::{JvmImplementation, RustJavaJvmImplementation};
use runtime::JvmRuntime;

pub static WIE_RUSTJAR: &str = "net.wie.rustjar";

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
        let jvm = Jvm::new(
            java_runtime::get_bootstrap_class_loader(Box::new(runtime.clone())),
            move || runtime.current_task_id(),
            properties,
        )
        .await
        .map_err(|x| WieError::FatalError(format!("Failed to create JVM: {x}")))?;

        Ok(jvm)
    }

    pub async fn to_wie_err(jvm: &Jvm, err: JavaError) -> WieError {
        match err {
            JavaError::JavaException(x) => {
                let string_writer = jvm.new_class("java/io/StringWriter", "()V", ()).await.unwrap();
                let print_writer = jvm
                    .new_class("java/io/PrintWriter", "(Ljava/io/Writer;)V", (string_writer.clone(),))
                    .await
                    .unwrap();

                let _: () = jvm
                    .invoke_virtual(&x, "printStackTrace", "(Ljava/io/PrintWriter;)V", (print_writer,))
                    .await
                    .unwrap();

                let trace = jvm.invoke_virtual(&string_writer, "toString", "()Ljava/lang/String;", []).await.unwrap();

                WieError::FatalError(format!("\n{}", JavaLangString::to_rust_string(jvm, &trace).await.unwrap()))
            }
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
