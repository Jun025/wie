use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

use test_utils::{TestClock, TestPlatform};
use wie_backend::{DefaultTaskRunner, System};
use wie_util::Result;

#[test]
fn sleep_waits_from_the_call_time_after_task_work() -> Result<()> {
    let clock = TestClock::new();
    clock.set(1000);
    let mut system = System::new(Box::new(TestPlatform::with_clock(clock.clone())), "test", "test", DefaultTaskRunner);
    let completed = Arc::new(AtomicBool::new(false));
    let task_system = system.clone();
    let task_clock = clock.clone();
    let task_completed = completed.clone();
    system.spawn(async move || {
        // Simulate work between the executor's poll and the sleep request.
        task_clock.advance(35);
        task_system.sleep(115).await;
        task_completed.store(true, Ordering::SeqCst);
        Ok(())
    });

    system.tick()?;
    for now in [1115, 1149] {
        clock.set(now);
        system.tick()?;
        assert!(!completed.load(Ordering::SeqCst), "sleep resumed early at {now}");
    }
    clock.set(1150);
    system.tick()?;
    assert!(completed.load(Ordering::SeqCst));
    Ok(())
}

#[test]
fn a_poll_counts_from_its_own_step_so_an_idle_tick_ends_after_a_long_one() -> Result<()> {
    // The event thread's poll is up 1ms after the step it is asked in started (`System::poll`): the
    // step, not the tick. A thread that spends 3ms of a step and then polls has a deadline already
    // behind it, so the next step polls it again — and that step is short, its poll is ahead, and
    // with every thread asleep the tick ends there. Counted from where the tick started, the
    // deadline stays behind for the rest of the tick and the thread is polled on every pass to the
    // budget's end: with every other test green, an idle game cost a 60Hz host 91-110% more CPU
    // (docs/report/0581 and its review; 0556 §11). The clock moves 10µs a read, so a tick that
    // spins shows in the polls it ran and in where it ended.
    for (beat_ms, margin, polls_in_the_tick, ends_at, host_wake) in [
        // No wake inside the 14ms budget: the tick ends as soon as the poll is ahead again.
        (100, None, 2, 1003, None),
        (100, Some(3), 2, 1003, None),
        // A wake inside it, on a host that comes back for one (`Platform::wake_margin_ms`): handed
        // over at that same step.
        (10, Some(3), 2, 1003, Some(1010)),
        // And on a host that does not: the tick waits for it itself, a poll every millisecond.
        (10, None, 9, 1010, None),
    ] {
        let clock = TestClock::stepping_micros(10);
        clock.set(1000);
        let platform = TestPlatform::with_clock(clock.clone());
        let platform = margin.into_iter().fold(platform, TestPlatform::with_wake_margin);
        let mut system = System::new(Box::new(platform), "test", "test", DefaultTaskRunner);
        let beat_system = system.clone();
        system.spawn(async move || {
            for _ in 0..1_000_000 {
                beat_system.sleep(beat_ms).await;
            }
            Ok(())
        });
        let polls = Arc::new(AtomicU64::new(0));
        let (poll_system, poll_clock, poll_count) = (system.clone(), clock.clone(), polls.clone());
        system.spawn(async move || {
            // The paint: 3ms of the first step, before the thread is back at its queue.
            poll_clock.advance(3);
            for _ in 0..1_000_000 {
                poll_count.fetch_add(1, Ordering::SeqCst);
                poll_system.poll(None, None).await;
            }
            Ok(())
        });

        system.tick()?;
        let case = format!("a {beat_ms}ms beat, margin {margin:?}");
        assert_eq!(polls.load(Ordering::SeqCst), polls_in_the_tick, "polls in the tick: {case}");
        assert_eq!(clock.peek(), ends_at, "where the tick ended: {case}");
        assert_eq!(system.pacing().wake().map(|x| x.raw()), host_wake, "the wake left to the host: {case}");
    }
    Ok(())
}
