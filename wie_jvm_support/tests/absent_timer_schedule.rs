//! `java.util.Timer.schedule(TimerTask, long)` — absent from `dlunch/RustJava@5b84dd1`, added back
//! by `hardening`. Trace-specified as method-not-found on 소울카드마스터2 back when the corpus
//! existed, so "resolves and does the right thing" is the whole assertion. No game files.
//!
//! An integration test rather than a unit test: it needs a concrete `TimerTask` subclass, so it
//! has to hand `run_jvm_test` a proto — and `test_utils` links the *library* build of
//! `wie_jvm_support`, so a proto built inside `#[cfg(test)]` carries a different `WieJvmContext`
//! type and will not unify. From out here both sides see the same crate.
//!
//! One test per file so that a crash names itself: `cargo tarpaulin` reports the failing *binary*,
//! never the test, so two JVM tests sharing a binary make a segfault unattributable (measured —
//! that is exactly how the coverage job went red on this branch).

use java_class_proto::JavaMethodProto;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

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
