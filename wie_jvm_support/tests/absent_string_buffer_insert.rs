//! `java.lang.StringBuffer.insert(int, String)` — absent from `dlunch/RustJava@5b84dd1`, added
//! back by `hardening`. Trace-specified as method-not-found on 미니고치 back when the corpus
//! existed, so "resolves and does the right thing" is the whole assertion. No game files.
//!
//! One test per file, and integration tests rather than unit tests — both on purpose, see
//! `absent_timer_schedule.rs` for why.

use jvm::{ClassInstanceRef, JavaError, runtime::JavaLangString};

use test_utils::run_jvm_test;
use wie_util::Result;

fn exception_class(err: JavaError) -> String {
    let JavaError::JavaException(instance) = err;

    instance.class_definition().name()
}

#[test]
fn string_buffer_insert() -> Result<()> {
    run_jvm_test(Box::new([]), |jvm| async move {
        let hello = JavaLangString::from_rust_string(&jvm, "hello!").await?;
        let buffer = jvm.new_class("java/lang/StringBuffer", "(Ljava/lang/String;)V", (hello,)).await?;

        let world = JavaLangString::from_rust_string(&jvm, " world").await?;
        let inserted: ClassInstanceRef<()> = jvm
            .invoke_virtual(&buffer, "insert", "(ILjava/lang/String;)Ljava/lang/StringBuffer;", (5, world))
            .await?;
        let text = jvm.invoke_virtual(&inserted, "toString", "()Ljava/lang/String;", ()).await?;
        assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, "hello world!");

        // spec: a null String inserts the four characters "null"
        let null_string: ClassInstanceRef<()> = None.into();
        let _: ClassInstanceRef<()> = jvm
            .invoke_virtual(&buffer, "insert", "(ILjava/lang/String;)Ljava/lang/StringBuffer;", (0, null_string))
            .await?;
        let text = jvm.invoke_virtual(&buffer, "toString", "()Ljava/lang/String;", ()).await?;
        assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, "nullhello world!");

        // spec says StringIndexOutOfBoundsException; this runtime does not define that class, so
        // the check throws its registered superclass rather than nothing at all.
        let x = JavaLangString::from_rust_string(&jvm, "x").await?;
        let err = jvm
            .invoke_virtual(&buffer, "insert", "(ILjava/lang/String;)Ljava/lang/StringBuffer;", (999, x))
            .await
            .map(|_: ClassInstanceRef<()>| ())
            .expect_err("insert past the end must throw");
        assert_eq!(exception_class(err), "java/lang/IndexOutOfBoundsException");

        Ok(())
    })
}
