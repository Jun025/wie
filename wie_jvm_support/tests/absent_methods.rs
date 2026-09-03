//! The two methods `dlunch/RustJava@5b84dd1` does not define and `hardening` adds back.
//!
//! An integration test rather than a unit test on purpose: it needs a concrete `TimerTask`
//! subclass, so it has to hand `run_jvm_test` a proto — and `test_utils` links the *library*
//! build of `wie_jvm_support`, so a proto built inside `#[cfg(test)]` carries a different
//! `WieJvmContext` type and will not unify. From out here both sides see the same crate.
//!
//! Each method was "trace-specified as method-not-found" on a real title back when the corpus
//! existed (`Timer.schedule` → 소울카드마스터2, `StringBuffer.insert` → 미니고치). Without them
//! the guest gets a resolution error at that call, so "resolves and does the right thing" is
//! the whole assertion. No game files are involved.

use java_class_proto::JavaMethodProto;
use jvm::{ClassInstanceRef, JavaError, Jvm, Result as JvmResult, runtime::JavaLangString};

use test_utils::run_jvm_test;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_util::Result;

/// `java.util.TimerTask.run` is abstract, so scheduling anything needs a concrete subclass.
fn one_shot_task_proto() -> WieJavaClassProto {
    WieJavaClassProto {
        name: "net/wie/test/OneShotTask",
        parent_class: Some("java/util/TimerTask"),
        interfaces: vec![],
        methods: vec![
            JavaMethodProto::new("<init>", "()V", one_shot_task_init, Default::default()),
            JavaMethodProto::new("run", "()V", one_shot_task_run, Default::default()),
        ],
        fields: vec![],
        access_flags: Default::default(),
    }
}

async fn one_shot_task_init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<()>) -> JvmResult<()> {
    let _: () = jvm.invoke_special(&this, "java/util/TimerTask", "<init>", "()V", ()).await?;

    Ok(())
}

async fn one_shot_task_run(_: &Jvm, _: &mut WieJvmContext, _: ClassInstanceRef<()>) -> JvmResult<()> {
    Ok(())
}

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

#[test]
fn timer_schedule_one_shot() -> Result<()> {
    let protos: Box<[WieJavaClassProto]> = Box::new([one_shot_task_proto()]);

    run_jvm_test(Box::new([protos]), |jvm| async move {
        let timer = jvm.new_class("java/util/Timer", "()V", ()).await?;
        let task = jvm.new_class("net/wie/test/OneShotTask", "()V", ()).await?;

        let _: () = jvm
            .invoke_virtual(&timer, "schedule", "(Ljava/util/TimerTask;J)V", (task.clone(), 10_000i64))
            .await?;

        // Asserted through the two fields `TimerThread` actually reads, so the test never waits on
        // a clock: `period > 0` is what makes a task repeat, and a one-shot must leave it 0.
        let period: i64 = jvm.get_field(&task, "period", "J").await?;
        assert_eq!(period, 0, "one-shot must leave period 0 — TimerThread repeats while period > 0");

        let next: i64 = jvm.get_field(&task, "nextExecutionTime", "J").await?;
        assert!(next >= 10_000, "delay must be added to now, got {next}");

        Ok(())
    })
}
