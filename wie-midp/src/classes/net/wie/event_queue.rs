use alloc::{string::ToString, vec, vec::Vec};

use futures::TryFutureExt;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::Runnable;

use wie_backend::{Event, KeyCode, YieldFuture};
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::midlet::MIDlet;

#[repr(i32)]
#[allow(clippy::enum_variant_names)]
enum EventQueueEvent {
    // TODO it's wipi event codes
    KeyEvent = 1,
    RepaintEvent = 41,
    NotifyEvent = 1000,
}

impl EventQueueEvent {
    fn from_raw(raw: i32) -> Option<Self> {
        Some(match raw {
            x if x == Self::KeyEvent as i32 => Self::KeyEvent,
            x if x == Self::RepaintEvent as i32 => Self::RepaintEvent,
            x if x == Self::NotifyEvent as i32 => Self::NotifyEvent,
            _ => return None,
        })
    }
}

#[repr(i32)]
#[derive(Debug)]
#[allow(dead_code, clippy::enum_variant_names)]
pub enum KeyboardEventType {
    KeyPressed = 1,
    KeyReleased = 2,
    KeyRepeated = 3,
    KeyTyped = 4,
}

impl KeyboardEventType {
    pub fn from_raw(raw: i32) -> Option<Self> {
        Some(match raw {
            x if x == Self::KeyPressed as i32 => Self::KeyPressed,
            x if x == Self::KeyReleased as i32 => Self::KeyReleased,
            x if x == Self::KeyRepeated as i32 => Self::KeyRepeated,
            x if x == Self::KeyTyped as i32 => Self::KeyTyped,
            _ => return None,
        })
    }
}

#[repr(i32)]
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum MIDPKeyCode {
    // keycode is for skvm
    UP = 141, // MIDP Canvas's name
    DOWN = 146,
    LEFT = 142,
    RIGHT = 145,
    FIRE = 148,
    LEFT_SOFT_KEY = 6,
    RIGHT_SOFT_KEY = 7,
    CLEAR = 8,
    CALL = 10,
    HANGUP = -1,
    VOLUME_UP = 13,
    VOLUME_DOWN = 14,

    KEY_NUM0 = 48,
    KEY_NUM1 = 49,
    KEY_NUM2 = 50,
    KEY_NUM3 = 51,
    KEY_NUM4 = 52,
    KEY_NUM5 = 53,
    KEY_NUM6 = 54,
    KEY_NUM7 = 55,
    KEY_NUM8 = 56,
    KEY_NUM9 = 57,
    KEY_POUND = 35, // #
    KEY_STAR = 42,  // *
}

impl MIDPKeyCode {
    pub fn from_raw(raw: i32) -> Option<Self> {
        Some(match raw {
            x if x == Self::UP as i32 => Self::UP,
            x if x == Self::DOWN as i32 => Self::DOWN,
            x if x == Self::LEFT as i32 => Self::LEFT,
            x if x == Self::RIGHT as i32 => Self::RIGHT,
            x if x == Self::FIRE as i32 => Self::FIRE,
            x if x == Self::LEFT_SOFT_KEY as i32 => Self::LEFT_SOFT_KEY,
            x if x == Self::RIGHT_SOFT_KEY as i32 => Self::RIGHT_SOFT_KEY,
            x if x == Self::CLEAR as i32 => Self::CLEAR,
            x if x == Self::CALL as i32 => Self::CALL,
            x if x == Self::HANGUP as i32 => Self::HANGUP,
            x if x == Self::VOLUME_UP as i32 => Self::VOLUME_UP,
            x if x == Self::VOLUME_DOWN as i32 => Self::VOLUME_DOWN,
            x if x == Self::KEY_NUM0 as i32 => Self::KEY_NUM0,
            x if x == Self::KEY_NUM1 as i32 => Self::KEY_NUM1,
            x if x == Self::KEY_NUM2 as i32 => Self::KEY_NUM2,
            x if x == Self::KEY_NUM3 as i32 => Self::KEY_NUM3,
            x if x == Self::KEY_NUM4 as i32 => Self::KEY_NUM4,
            x if x == Self::KEY_NUM5 as i32 => Self::KEY_NUM5,
            x if x == Self::KEY_NUM6 as i32 => Self::KEY_NUM6,
            x if x == Self::KEY_NUM7 as i32 => Self::KEY_NUM7,
            x if x == Self::KEY_NUM8 as i32 => Self::KEY_NUM8,
            x if x == Self::KEY_NUM9 as i32 => Self::KEY_NUM9,
            x if x == Self::KEY_POUND as i32 => Self::KEY_POUND,
            x if x == Self::KEY_STAR as i32 => Self::KEY_STAR,
            _ => return None,
        })
    }

    fn from_key_code(keycode: KeyCode) -> Self {
        match keycode {
            KeyCode::UP => Self::UP,
            KeyCode::DOWN => Self::DOWN,
            KeyCode::LEFT => Self::LEFT,
            KeyCode::RIGHT => Self::RIGHT,
            KeyCode::OK => Self::FIRE,
            KeyCode::LEFT_SOFT_KEY => Self::LEFT_SOFT_KEY,
            KeyCode::RIGHT_SOFT_KEY => Self::RIGHT_SOFT_KEY,
            KeyCode::CLEAR => Self::CLEAR,
            KeyCode::CALL => Self::CALL,
            KeyCode::HANGUP => Self::HANGUP,
            KeyCode::VOLUME_UP => Self::VOLUME_UP,
            KeyCode::VOLUME_DOWN => Self::VOLUME_DOWN,
            KeyCode::NUM0 => Self::KEY_NUM0,
            KeyCode::NUM1 => Self::KEY_NUM1,
            KeyCode::NUM2 => Self::KEY_NUM2,
            KeyCode::NUM3 => Self::KEY_NUM3,
            KeyCode::NUM4 => Self::KEY_NUM4,
            KeyCode::NUM5 => Self::KEY_NUM5,
            KeyCode::NUM6 => Self::KEY_NUM6,
            KeyCode::NUM7 => Self::KEY_NUM7,
            KeyCode::NUM8 => Self::KEY_NUM8,
            KeyCode::NUM9 => Self::KEY_NUM9,
            KeyCode::HASH => Self::KEY_POUND,
            KeyCode::STAR => Self::KEY_STAR,
        }
    }
}

// class net.wie.EventQueue
pub struct EventQueue;

impl EventQueue {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/EventQueue",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getNextEvent", "([I)V", Self::get_next_event, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("dispatchEvent", "([I)V", Self::dispatch_event, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("callSerially", "(Ljava/lang/Runnable;)V", Self::call_serially, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("dispatchCallbacks", "()V", Self::dispatch_callbacks, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "getEventQueue",
                    "()Lnet/wie/EventQueue;",
                    Self::get_event_queue,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("eventQueue", "Lnet/wie/EventQueue;", FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC),
                JavaFieldProto::new("callSeriallyEvents", "Ljava/util/Vector;", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.EventQueue::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        let call_serially_events = jvm.new_class("java/util/Vector", "()V", ()).await?;
        jvm.put_field(
            &mut this,
            "net/wie/EventQueue",
            "callSeriallyEvents",
            "Ljava/util/Vector;",
            call_serially_events,
        )
        .await?;

        Ok(())
    }

    // TODO this resembles WIPI's architecture for now, but we need to change it to event listener
    async fn get_next_event(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        mut event: ClassInstanceRef<Array<i32>>,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.EventQueue::getNextEvent({this:?}, {event:?})");

        let mut pending_timer_events = Vec::new();
        loop {
            let now = context.system().platform().now();
            let maybe_event = context.system().event_queue().pop();

            if let Some(x) = maybe_event {
                let event_data = match x {
                    Event::Redraw => vec![EventQueueEvent::RepaintEvent as _, 0, 0, 0],
                    Event::Keydown(x) => vec![
                        EventQueueEvent::KeyEvent as _,
                        KeyboardEventType::KeyPressed as _,
                        MIDPKeyCode::from_key_code(x) as _,
                        0,
                    ],
                    Event::Keyup(x) => vec![
                        EventQueueEvent::KeyEvent as _,
                        KeyboardEventType::KeyReleased as _,
                        MIDPKeyCode::from_key_code(x) as _,
                        0,
                    ],
                    Event::Keyrepeat(x) => vec![
                        EventQueueEvent::KeyEvent as _,
                        KeyboardEventType::KeyRepeated as _,
                        MIDPKeyCode::from_key_code(x) as _,
                        0,
                    ],
                    Event::Timer { due, pace_from, callback } => {
                        // TODO we should wait for timer more efficiently
                        if due <= now {
                            context.system().pacing().timer_fired(now - due);
                            callback()
                                .or_else(async |x| Err(jvm.exception("net/wie/WieError", &x.to_string()).await))
                                .await?
                        } else {
                            // push it to event queue again
                            pending_timer_events.push(Event::Timer { due, pace_from, callback });
                        }

                        continue;
                    }
                    // wipi notifyEvent
                    Event::Notify { r#type, param1, param2 } => vec![EventQueueEvent::NotifyEvent as i32, r#type, param1, param2],
                };

                jvm.store_array(&mut event, 0, event_data).await?;

                break;
            } else {
                let events = jvm
                    .get_field(&this, "net/wie/EventQueue", "callSeriallyEvents", "Ljava/util/Vector;")
                    .await?;
                let count: i32 = jvm.invoke_virtual(&events, "java/util/Vector", "size", "()I", ()).await?;
                if count > 0 {
                    let midlet: ClassInstanceRef<MIDlet> = jvm
                        .get_static_field("javax/microedition/midlet/MIDlet", "currentMIDlet", "Ljavax/microedition/midlet/MIDlet;")
                        .await?;
                    if !midlet.is_null() {
                        let display = MIDlet::display(jvm, &midlet).await?;
                        // A frontend Redraw may not have reached the backend queue yet.
                        let _: () = jvm
                            .invoke_virtual(&display, "javax/microedition/lcdui/Display", "serviceRepaints", "()V", ())
                            .await?;
                    }
                }
                Self::dispatch_callbacks(jvm, context, this.clone()).await?;

                // Wake for the earliest pending timer instead of sleeping a whole slice: a timer
                // that falls due mid-slice otherwise waits for the next one, which costs a
                // timer-paced guest (a KTF clet's game loop) one extra host frame per game frame.
                // Recurring callSerially work keeps the 16ms slice it is paced by.
                let callbacks_pending = jvm.invoke_virtual::<_, i32>(&events, "java/util/Vector", "size", "()I", ()).await? > 0;
                let earliest_timer = pending_timer_events
                    .iter()
                    .filter_map(|x| match x {
                        Event::Timer { due, .. } => Some(*due),
                        _ => None,
                    })
                    .min();
                let next_timer = earliest_timer.map(|x| x.raw().saturating_sub(now.raw())).filter(|_| !callbacks_pending);
                // Re-check the backend queue every millisecond of the wait: a Redraw handed over
                // mid-wait otherwise sits until the wait ends, and 배틀몬스터's game thread spins until
                // its paint lands (measured 16.4ms of a 50ms frame). Callbacks and timers still run
                // once per wait, as before.
                let deadline = now + next_timer.map_or(16, |x| x.clamp(1, 16));
                // Those 1ms sleeps are polls, and a tick does not stay alive for a poll — so a timer
                // due mid-tick fired on the host's next frame (16.7ms grid; 18 titles, 2026-09-28
                // census). Poll on behalf of the earliest timer instead, as a sleep until it would —
                // if it may keep this tick alive (`Event::guest_timer`). Asked every poll: the wait
                // outlives the tick, and a timer held back in this one may keep the next alive.
                let pace = |tick| {
                    pending_timer_events
                        .iter()
                        .filter_map(|x| match x {
                            Event::Timer { due, pace_from, .. } if *pace_from <= tick => Some(*due),
                            _ => None,
                        })
                        .min()
                        .filter(|_| !callbacks_pending)
                };
                // What this wait is for, for `AchievedSpeed`: a host that does not tick again before
                // it is the game running late — a held-back timer included, which `pace` leaves out.
                // And the first of its polls that finds work (`System::poll_until`).
                let due = if callbacks_pending { Some(deadline) } else { earliest_timer };
                context.system().pacing().guest_waiting_until(due);
                // One clock read per poll: the poll itself counts from its step's start (`System::poll`).
                while context.system().platform().now() < deadline {
                    let tick = context.system().pacing().ticks();
                    context.system().poll(pace(tick), due).await;
                    if !context.system().event_queue().is_empty() {
                        break;
                    }
                }

                context.system().pacing().guest_waiting_until(None);

                for event in pending_timer_events.drain(..) {
                    context.system().event_queue().push(event);
                }
            }
        }

        for event in pending_timer_events {
            context.system().event_queue().push(event);
        }

        Ok(())
    }

    async fn dispatch_event(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        event: ClassInstanceRef<Array<i32>>,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.EventQueue::dispatchEvent({this:?}, {event:?})");

        let current_midlet: ClassInstanceRef<MIDlet> = jvm
            .get_static_field("javax/microedition/midlet/MIDlet", "currentMIDlet", "Ljavax/microedition/midlet/MIDlet;")
            .await?;

        let display = jvm
            .invoke_static(
                "javax/microedition/lcdui/Display",
                "getDisplay",
                "(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;",
                (current_midlet,),
            )
            .await?;

        let event = jvm.load_array(&event, 0, 4).await?;
        let event_kind = if let Some(event_kind) = EventQueueEvent::from_raw(event[0]) {
            event_kind
        } else {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "Invalid event queue event type")
                .await);
        };

        match event_kind {
            EventQueueEvent::RepaintEvent => {
                let _: () = jvm
                    .invoke_virtual(&display, "javax/microedition/lcdui/Display", "handlePaintEvent", "()V", ())
                    .await?;
                Self::dispatch_callbacks(jvm, context, this).await?;
            }
            EventQueueEvent::KeyEvent => {
                let event_type = if let Some(event_type) = KeyboardEventType::from_raw(event[1]) {
                    event_type
                } else {
                    return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid keyboard event type").await);
                };
                let code = event[2];

                let _: () = jvm
                    .invoke_virtual(
                        &display,
                        "javax/microedition/lcdui/Display",
                        "handleKeyEvent",
                        "(II)V",
                        (event_type as i32, code),
                    )
                    .await?;
            }
            EventQueueEvent::NotifyEvent => {
                let r#type = event[1];
                let param1 = event[2];
                let param2 = event[3];

                let _: () = jvm
                    .invoke_virtual(
                        &display,
                        "javax/microedition/lcdui/Display",
                        "handleNotifyEvent",
                        "(III)V",
                        (r#type, param1, param2),
                    )
                    .await?;
            }
        }

        Ok(())
    }

    async fn get_event_queue(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("net.wie.EventQueue::getEventQueue()");

        let event_queue: ClassInstanceRef<Self> = jvm.get_static_field("net/wie/EventQueue", "eventQueue", "Lnet/wie/EventQueue;").await?;
        let event_queue = if event_queue.is_null() {
            let instance = jvm.new_class("net/wie/EventQueue", "()V", ()).await?;
            jvm.put_static_field("net/wie/EventQueue", "eventQueue", "Lnet/wie/EventQueue;", instance.clone())
                .await?;

            instance.into()
        } else {
            event_queue
        };

        Ok(event_queue)
    }

    async fn call_serially(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        event: ClassInstanceRef<Runnable>,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.EventQueue::callSerially({this:?}, {event:?})");

        let call_serially_events = jvm
            .get_field(&this, "net/wie/EventQueue", "callSeriallyEvents", "Ljava/util/Vector;")
            .await?;
        jvm.invoke_virtual(&call_serially_events, "java/util/Vector", "addElement", "(Ljava/lang/Object;)V", (event,))
            .await
    }

    async fn dispatch_callbacks(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let events = jvm
            .get_field(&this, "net/wie/EventQueue", "callSeriallyEvents", "Ljava/util/Vector;")
            .await?;
        let count: i32 = jvm.invoke_virtual(&events, "java/util/Vector", "size", "()I", ()).await?;
        // Callbacks registered during this batch belong to the next event-loop turn.
        for _ in 0..count {
            let callback: ClassInstanceRef<Runnable> = jvm
                .invoke_virtual(&events, "java/util/Vector", "remove", "(I)Ljava/lang/Object;", (0,))
                .await?;
            let _: () = jvm.invoke_virtual(&callback, "java/lang/Runnable", "run", "()V", ()).await?;
            YieldFuture::new().await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, sync::Arc, vec, vec::Vec};
    use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

    use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
    use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

    use test_utils::{TestClock, TestPlatform, run_jvm_test_with_system};
    use wie_backend::{DefaultTaskRunner, Event, Instant, KeyCode, SpeedClock, System, YieldFuture};
    use wie_jvm_support::{JvmSupport, RustJavaJvmImplementation, WieJavaClassProto, WieJvmContext};
    use wie_util::Result;

    use crate::{
        classes::javax::microedition::{
            lcdui::{Display, Graphics, Image},
            midlet::MIDlet,
        },
        get_protos,
    };

    use super::{EventQueue, EventQueueEvent};

    struct IdleCallback;

    impl IdleCallback {
        async fn run(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
            let count: i32 = jvm.get_field(&this, "net/wie/IdleCallback", "count", "I").await?;
            let now = context.system().platform().now().raw() as i64;
            if count > 0 {
                let last_run: i64 = jvm.get_field(&this, "net/wie/IdleCallback", "lastRun", "J").await?;
                assert!(now - last_run >= 16, "idle callbacks must retain the timed wait");
                context.system().event_queue().push(Event::Keydown(KeyCode::NUM1));
            } else {
                let queue = jvm
                    .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&queue, "net/wie/EventQueue", "callSerially", "(Ljava/lang/Runnable;)V", (this.clone(),))
                    .await?;
            }
            jvm.put_field(&mut this, "net/wie/IdleCallback", "count", "I", count + 1).await?;
            jvm.put_field(&mut this, "net/wie/IdleCallback", "lastRun", "J", now).await
        }
    }

    #[test]
    fn recurring_callback_waits_while_the_backend_queue_is_empty() -> Result<()> {
        let callback_proto = WieJavaClassProto {
            name: "net/wie/IdleCallback",
            parent_class: Some("java/lang/Object"),
            interfaces: vec!["java/lang/Runnable"],
            methods: vec![JavaMethodProto::new("run", "()V", IdleCallback::run, MethodAccessFlags::PUBLIC)],
            fields: vec![
                JavaFieldProto::new("count", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("lastRun", "J", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        };
        let clock = TestClock::new();
        let mut system = System::new(Box::new(TestPlatform::with_clock(clock.clone())), "", "", DefaultTaskRunner);
        let completed = Arc::new(AtomicBool::new(false));
        let completed_task = completed.clone();
        let system_task = system.clone();
        let clock_task = clock.clone();
        system.spawn(async move || {
            let jvm = JvmSupport::new_jvm(
                &system_task,
                None,
                Box::new([get_protos().into(), Box::new([callback_proto])]),
                &[],
                RustJavaJvmImplementation,
            )
            .await?;
            let queue = jvm
                .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
                .await
                .unwrap();
            let callback = jvm.instantiate_class("net/wie/IdleCallback").await.unwrap();
            let _: () = jvm
                .invoke_virtual(&queue, "net/wie/EventQueue", "callSerially", "(Ljava/lang/Runnable;)V", (callback,))
                .await
                .unwrap();
            let event = jvm.instantiate_array("I", 4).await.unwrap();
            let _: () = jvm
                .invoke_virtual(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event,))
                .await
                .unwrap();
            completed_task.store(true, Ordering::SeqCst);
            // Let the enclosing tick finish after this task completes.
            clock_task.advance(100);
            Ok(())
        });

        system.tick()?;
        assert!(!completed.load(Ordering::SeqCst));
        clock.set(15);
        system.tick()?;
        assert!(!completed.load(Ordering::SeqCst));
        clock.set(16);
        system.tick()?;
        clock.set(32);
        system.tick()?;
        assert!(completed.load(Ordering::SeqCst));
        Ok(())
    }

    // Runs `getNextEvent` once on a fresh JVM whose backend queue holds a timer due at 5ms, with
    // the clock held at `before` and then at 5. Returns whether the call returned at each point.
    fn get_next_event_with_timer_due_at_5ms(
        before: u64,
        extra_protos: Vec<WieJavaClassProto>,
        recurring_callback: Option<&'static str>,
    ) -> Result<(bool, bool)> {
        let clock = TestClock::new();
        let mut system = System::new(Box::new(TestPlatform::with_clock(clock.clone())), "", "", DefaultTaskRunner);
        let timer_system = system.clone();
        system.event_queue().push(Event::timer(Instant::from_epoch_millis(5), move || async move {
            // The key is what makes getNextEvent return; the timer itself is consumed internally.
            timer_system.event_queue().push(Event::Keydown(KeyCode::NUM1));
            Ok(())
        }));
        let completed = Arc::new(AtomicBool::new(false));
        let completed_task = completed.clone();
        let system_task = system.clone();
        let clock_task = clock.clone();
        system.spawn(async move || {
            let jvm = JvmSupport::new_jvm(
                &system_task,
                None,
                Box::new([get_protos().into(), extra_protos.into()]),
                &[],
                RustJavaJvmImplementation,
            )
            .await?;
            let queue = jvm
                .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
                .await
                .unwrap();
            if let Some(class) = recurring_callback {
                let callback = jvm.instantiate_class(class).await.unwrap();
                let _: () = jvm
                    .invoke_virtual(&queue, "net/wie/EventQueue", "callSerially", "(Ljava/lang/Runnable;)V", (callback,))
                    .await
                    .unwrap();
            }
            let event = jvm.instantiate_array("I", 4).await.unwrap();
            let _: () = jvm
                .invoke_virtual(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event,))
                .await
                .unwrap();
            completed_task.store(true, Ordering::SeqCst);
            clock_task.advance(100);
            Ok(())
        });

        system.tick()?;
        clock.set(before);
        system.tick()?;
        let at_before = completed.load(Ordering::SeqCst);
        clock.set(5);
        system.tick()?;
        Ok((at_before, completed.load(Ordering::SeqCst)))
    }

    // `Pacing::due` after one tick of a fresh `System` whose task `guest` runs on a clock held at 0:
    // what `AchievedSpeed` (wie-backend speed_clock.rs) reads to tell a guest that has work due from
    // one waiting for a key. `guest` gets the system and, for a JVM guest, a JVM to call into.
    fn due_after_a_tick<F, Fut>(guest: F) -> Result<Option<u64>>
    where
        F: FnOnce(System, Jvm) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let clock = TestClock::new();
        let mut system = System::new(Box::new(TestPlatform::with_clock(clock)), "", "", DefaultTaskRunner);
        let system_task = system.clone();
        system.spawn(async move || {
            let jvm = JvmSupport::new_jvm(&system_task, None, Box::new([get_protos().into()]), &[], RustJavaJvmImplementation).await?;
            guest(system_task, jvm).await;
            Ok(())
        });
        system.tick()?;
        let due = system.pacing().due().map(|x| x.raw());
        Ok(due)
    }

    async fn get_next_event(jvm: Jvm) {
        next_event_kind(jvm).await;
    }

    // `getNextEvent`, and the kind of the event it handed back (`EventQueueEvent`).
    async fn next_event_kind(jvm: Jvm) -> i32 {
        let queue = jvm
            .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
            .await
            .unwrap();
        let event: ClassInstanceRef<Array<i32>> = jvm.instantiate_array("I", 4).await.unwrap().into();
        let _: () = jvm
            .invoke_virtual(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event.clone(),))
            .await
            .unwrap();
        jvm.load_array::<i32>(&event, 0, 1).await.unwrap()[0]
    }

    #[test]
    fn a_tick_reports_a_sleeping_guest_s_wake_as_due_and_a_poll_as_nothing() -> Result<()> {
        assert_eq!(due_after_a_tick(|system, _| async move { system.sleep(10).await })?, Some(10));
        // A 1ms sleep is a poll (`POLL_SLEEP_MS`): it says when the guest looks again, not that it has
        // work then. Counted, every guest that polls would read as running late on any host.
        assert_eq!(
            due_after_a_tick(|system, _| async move {
                loop {
                    system.sleep(1).await
                }
            })?,
            None
        );
        Ok(())
    }

    #[test]
    fn a_guest_waiting_for_a_key_has_nothing_due() -> Result<()> {
        // getNextEvent on an empty queue waits a 16ms slice in 1ms polls: nothing is due, so a slow
        // host does not make a menu waiting for a key read as running late.
        assert_eq!(due_after_a_tick(|_, jvm| get_next_event(jvm))?, None);
        Ok(())
    }

    #[test]
    fn a_held_back_timer_is_due_though_the_event_loop_only_polls_for_it() -> Result<()> {
        // A timer that may not keep a tick alive (`Event::guest_timer`'s `pace_from` — here never) is
        // waited for in 1ms polls, which the executor does not count; only the event loop knows the
        // guest has work at 30.
        let due = due_after_a_tick(|system, jvm| async move {
            system
                .event_queue()
                .push(Event::guest_timer(Instant::from_epoch_millis(30), u64::MAX, || async { Ok(()) }));
            get_next_event(jvm).await
        })?;
        assert_eq!(due, Some(30));
        Ok(())
    }

    #[test]
    fn timer_due_mid_slice_fires_without_waiting_for_the_whole_slice() -> Result<()> {
        // A KTF clet paces its game loop with MC_knlSetTimer. With a flat 16ms sleep a timer due
        // at 5ms fired at 16ms, one host frame late on every game frame — half the frame rate.
        let (at_4ms, at_5ms) = get_next_event_with_timer_due_at_5ms(4, vec![], None)?;
        assert!(!at_4ms, "the timer must not fire early");
        assert!(at_5ms, "a timer due mid-slice must fire when it falls due");
        Ok(())
    }

    // A guest that arms `rounds` WIPI timers `after_ms` ahead, one per `getNextEvent`, on a host
    // that calls `tick` once per 17ms frame: the clock jumps to the next frame between ticks and
    // steps 10µs per engine read inside one, so the guest's own work costs well under a
    // millisecond and what is measured is the wait. Each round spends `work_ms` of guest work before
    // arming, as a timer callback does. `pace_after`: each timer may keep a tick alive from this many
    // ticks after the one it is armed in (`Event::guest_timer`), never if `None`. Returns the worst
    // lateness in ms.
    fn guest_timers_on_a_frame_grid(after_ms: u64, work_ms: u64, rounds: u64, pace_after: Option<u64>) -> Result<u64> {
        const FRAME_MS: u64 = 17;
        let clock = TestClock::stepping_micros(10);
        let mut system = System::new(Box::new(TestPlatform::with_clock(clock.clone())), "", "", DefaultTaskRunner);
        let worst = Arc::new(AtomicU64::new(0));
        let done = Arc::new(AtomicBool::new(false));
        let (system_task, clock_task, worst_task, done_task) = (system.clone(), clock.clone(), worst.clone(), done.clone());
        system.spawn(async move || {
            let jvm = JvmSupport::new_jvm(&system_task, None, Box::new([get_protos().into()]), &[], RustJavaJvmImplementation).await?;
            let queue = jvm
                .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
                .await
                .unwrap();
            for _ in 0..rounds {
                clock_task.advance(work_ms);
                let due = system_task.platform().now() + after_ms;
                let (system_timer, clock_timer, worst_timer) = (system_task.clone(), clock_task.clone(), worst_task.clone());
                system_task.event_queue().push(Event::guest_timer(
                    due,
                    pace_after.map_or(u64::MAX, |x| system_task.pacing().ticks() + x),
                    move || async move {
                        worst_timer.fetch_max(clock_timer.peek() - due.raw(), Ordering::SeqCst);
                        // The key is what makes getNextEvent return; the timer itself is consumed internally.
                        system_timer.event_queue().push(Event::Keydown(KeyCode::NUM1));
                        Ok(())
                    },
                ));
                let event = jvm.instantiate_array("I", 4).await.unwrap();
                let _: () = jvm
                    .invoke_virtual(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event,))
                    .await
                    .unwrap();
            }
            done_task.store(true, Ordering::SeqCst);
            Ok(())
        });

        let mut frames = 0;
        while !done.load(Ordering::SeqCst) {
            system.tick()?;
            clock.set((clock.peek() / FRAME_MS + 1) * FRAME_MS);
            frames += 1;
            assert!(frames < 10_000, "the guest never finished its timers");
        }
        Ok(worst.load(Ordering::SeqCst))
    }

    #[test]
    fn a_guest_timer_fires_on_time_not_on_the_next_host_frame() -> Result<()> {
        // 2026-09-28 census: 18 titles' WIPI timers fired up to a host frame late (p95 ≤ 23ms), because
        // the wait for them was 1ms polls and a tick does not stay alive for a poll. A 25ms period
        // walks across the 17ms grid; only a due in the ~3ms a tick leaves the host may wait for
        // the next frame.
        let worst = guest_timers_on_a_frame_grid(25, 0, 20, Some(0))?;
        assert!(worst <= 4, "a 25ms timer fired {worst}ms late");
        Ok(())
    }

    #[test]
    fn a_timer_held_back_in_its_arming_tick_fires_on_time_in_the_next() -> Result<()> {
        // The usual loop: a 10ms KTF clet timer re-armed after 8ms of its own callback's work may not
        // keep that tick alive, but its due lands just inside the next one — which must wait for it
        // rather than keep the verdict it got in the tick before (4b8c8f5ff7d6: 39 fps, not its 60).
        // An 18ms period walks across the 17ms grid, so some dues still land in the ~3ms a tick leaves
        // the host (measured worst 7); a stale verdict costs the whole next frame (16).
        let worst = guest_timers_on_a_frame_grid(10, 8, 20, Some(1))?;
        assert!(worst <= 8, "a timer paced from the next tick fired {worst}ms late");
        Ok(())
    }

    #[test]
    fn a_1ms_guest_timer_is_left_to_the_host_frame() -> Result<()> {
        // A poll (KTF 49ade89578c5's MC_knlSetTimer(1), or a timer re-armed in the tick it fired in)
        // keeps no tick alive: waiting on 49ade89578c5's ran it at 44fps, not its 39.
        let worst = guest_timers_on_a_frame_grid(25, 0, 20, None)?;
        assert!(worst > 4, "a timer marked a poll kept the tick alive (worst {worst}ms late)");
        Ok(())
    }

    // What runs on the host of `on_a_host`. `timer`: a WIPI timer the guest re-arms that many ms
    // ahead on every fire, with `pace_after` as in `guest_timers_on_a_frame_grid`. `beat_ms`: a thread
    // that sleeps on that beat. `poller`: a thread in a `sleep(1)` loop — a poll (`POLL_SLEEP_MS`).
    // `repaint`: whoever has the beat — the thread, or without one the timer's callback — asks for a
    // repaint on every beat, and the event loop counts the Redraws it is handed. `call_serially`: a
    // `callSerially` chain on the event loop, each run queueing itself again.
    #[derive(Clone, Copy, Debug, Default)]
    struct Guest {
        timer: Option<(u64, Option<u64>)>,
        beat_ms: Option<u64>,
        poller: bool,
        repaint: Option<Repaint>,
        call_serially: bool,
    }

    // The two ways a guest asks for a repaint.
    #[derive(Clone, Copy, Debug)]
    enum Repaint {
        // `Display.repaint`: the engine hands the Redraw on (`System::request_redraw`).
        Engine,
        // WIPI-C `MC_grpRepaint`: the host does, when it finds the screen's flag set (`Screen::request_redraw`).
        Screen,
    }

    impl Repaint {
        fn request(self, system: &System) {
            match self {
                Self::Engine => system.request_redraw(),
                Self::Screen => system.platform().screen().request_redraw().unwrap(),
            }
        }
    }

    // What `on_a_host` counted: timer fires and poller rounds, beats, Redraws handed to the event
    // loop — in all, the most in one host frame, and the most frames one took to get there from the
    // repaint that asked for it — `callSerially` runs, the host's wake ticks, the host frames the
    // engine counted (`Pacing::ticks`), and the guest ms the 120 frames took.
    #[derive(Debug, PartialEq)]
    struct HostRun {
        fires: u64,
        beats: u64,
        redraws: u64,
        redraws_in_a_frame: u64,
        redraw_frames_late: u64,
        callbacks: u64,
        wake_ticks: u64,
        ticks: u64,
        elapsed: u64,
    }

    // Which host frame each repaint was asked for in, and its Redraw handed to the event loop in.
    #[derive(Default)]
    struct RedrawLog {
        // The host frame now, from 1.
        frame: AtomicU64,
        // The frame of the oldest repaint whose Redraw has not come yet; 0 for none.
        asked: AtomicU64,
        // The frame of the last Redraw, and how many that frame has had.
        last: AtomicU64,
        in_last: AtomicU64,
        most_in_a_frame: AtomicU64,
        most_frames_late: AtomicU64,
    }

    impl RedrawLog {
        fn asked(&self) {
            let _ = self
                .asked
                .compare_exchange(0, self.frame.load(Ordering::SeqCst), Ordering::SeqCst, Ordering::SeqCst);
        }

        fn handed_on(&self) {
            let frame = self.frame.load(Ordering::SeqCst);
            if self.last.swap(frame, Ordering::SeqCst) != frame {
                self.in_last.store(0, Ordering::SeqCst);
            }
            let in_frame = self.in_last.fetch_add(1, Ordering::SeqCst) + 1;
            self.most_in_a_frame.fetch_max(in_frame, Ordering::SeqCst);
            let asked = self.asked.swap(0, Ordering::SeqCst);
            if asked != 0 {
                self.most_frames_late.fetch_max(frame - asked, Ordering::SeqCst);
            }
        }
    }

    extern crate std;

    // `ChainedCallback` runs so far. Per thread: a guest runs on the thread of the test that ticks it.
    std::thread_local! {
        static CALLBACK_RUNS: core::cell::Cell<u64> = const { core::cell::Cell::new(0) };
    }

    struct ChainedCallback;

    impl ChainedCallback {
        async fn run(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            CALLBACK_RUNS.with(|runs| runs.set(runs.get() + 1));
            let queue = jvm
                .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
                .await?;
            jvm.invoke_virtual(&queue, "net/wie/EventQueue", "callSerially", "(Ljava/lang/Runnable;)V", (this,))
                .await
        }
    }

    // `guest` over 120 frames of a 60Hz host at play `speed`. The test clock is the guest clock, so a
    // host frame is 16.67 x `speed` of its ms and the tick budget 14 x `speed` (`SpeedClock::budget`).
    // `margin`: `None` is a host that ticks once per frame. `Some(ms)` is the host of
    // `Platform::wake_margin_ms`, as the browser host runs it (`wie_featurephone`'s crate doc and
    // `advance`): after every tick it takes the pacing counters, and for `Pacing::wake` it comes
    // back `SpeedClock::wake_delay` later with a wake tick — unless the next frame comes first —
    // on a timer that fires no sooner than `timer_floor_ms` (a browser's is 1ms, 4ms once nested).
    // Either host hands the screen's redraw flag back as an `Event::Redraw` where `advance` does:
    // after a tick it does not come back for, or failing that before the next frame's.
    fn on_a_host(guest: Guest, speed: f64, margin: Option<u64>, timer_floor_ms: u64) -> Result<HostRun> {
        on_a_host_with_timers(guest, speed, margin, &[timer_floor_ms])
    }

    // `on_a_host` with a timer floor for each frame in turn: one longer than a frame is a frame the
    // host's timer loses to the next, so it does not come back in it.
    fn on_a_host_with_timers(guest: Guest, speed: f64, margin: Option<u64>, timer_floors_ms: &[u64]) -> Result<HostRun> {
        let frame_ms = (16.667 * speed).round() as u64;
        let budget = (14.0 * speed) as u64;
        let clock = TestClock::stepping_micros(10);
        let platform = TestPlatform::with_clock(clock.clone()).with_speed(speed);
        let platform = margin.into_iter().fold(platform, TestPlatform::with_wake_margin);
        let in_wake_tick = platform.wake_tick_flag();
        let screen_redraw = platform.redraw_flag();
        let mut system = System::new(Box::new(platform), "", "", DefaultTaskRunner);
        let fires = Arc::new(AtomicU64::new(0));
        let beats = Arc::new(AtomicU64::new(0));
        let redraws = Arc::new(AtomicU64::new(0));
        let log = Arc::new(RedrawLog::default());
        CALLBACK_RUNS.with(|runs| runs.set(0));
        if guest.timer.is_some() || guest.repaint.is_some() || guest.call_serially {
            let (system_task, fires_task, beats_task, redraws_task) = (system.clone(), fires.clone(), beats.clone(), redraws.clone());
            let log_task = log.clone();
            let callback_proto = WieJavaClassProto {
                name: "net/wie/ChainedCallback",
                parent_class: Some("java/lang/Object"),
                interfaces: vec!["java/lang/Runnable"],
                methods: vec![JavaMethodProto::new("run", "()V", ChainedCallback::run, MethodAccessFlags::PUBLIC)],
                fields: vec![],
                access_flags: ClassAccessFlags::PUBLIC,
            };
            system.spawn(async move || {
                let jvm = JvmSupport::new_jvm(
                    &system_task,
                    None,
                    Box::new([get_protos().into(), Box::new([callback_proto])]),
                    &[],
                    RustJavaJvmImplementation,
                )
                .await?;
                if guest.call_serially {
                    let queue = jvm
                        .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
                        .await
                        .unwrap();
                    let callback = jvm.instantiate_class("net/wie/ChainedCallback").await.unwrap();
                    let _: () = jvm
                        .invoke_virtual(&queue, "net/wie/EventQueue", "callSerially", "(Ljava/lang/Runnable;)V", (callback,))
                        .await
                        .unwrap();
                }
                for _ in 0..1_000_000 {
                    if let Some((after_ms, pace_after)) = guest.timer {
                        let due = system_task.platform().now() + after_ms;
                        let (system_timer, fires_timer, beats_timer) = (system_task.clone(), fires_task.clone(), beats_task.clone());
                        let log_timer = log_task.clone();
                        system_task.event_queue().push(Event::guest_timer(
                            due,
                            pace_after.map_or(u64::MAX, |x| system_task.pacing().ticks() + x),
                            move || async move {
                                fires_timer.fetch_add(1, Ordering::SeqCst);
                                if let (Some(repaint), None) = (guest.repaint, guest.beat_ms) {
                                    beats_timer.fetch_add(1, Ordering::SeqCst);
                                    log_timer.asked();
                                    repaint.request(&system_timer);
                                }
                                // The key is what makes getNextEvent return; the timer itself is consumed internally.
                                system_timer.event_queue().push(Event::Keydown(KeyCode::NUM1));
                                Ok(())
                            },
                        ));
                    }
                    // A Redraw is counted and the wait goes on: it is the timer's key that ends it.
                    while next_event_kind(jvm.clone()).await == EventQueueEvent::RepaintEvent as i32 {
                        redraws_task.fetch_add(1, Ordering::SeqCst);
                        log_task.handed_on();
                    }
                }
                Ok(())
            });
        }
        if guest.poller {
            let (system_poller, fires_poller) = (system.clone(), fires.clone());
            system.spawn(async move || {
                for _ in 0..1_000_000 {
                    fires_poller.fetch_add(1, Ordering::SeqCst);
                    system_poller.sleep(1).await;
                }
                Ok(())
            });
        }
        if let Some(beat_ms) = guest.beat_ms {
            let (system_beat, beats_beat, log_beat) = (system.clone(), beats.clone(), log.clone());
            system.spawn(async move || {
                for _ in 0..1_000_000 {
                    beats_beat.fetch_add(1, Ordering::SeqCst);
                    if let Some(repaint) = guest.repaint {
                        log_beat.asked();
                        repaint.request(&system_beat);
                    }
                    system_beat.sleep(beat_ms).await;
                }
                Ok(())
            });
        }

        // Guest ms = wall ms x `speed` from 0 on.
        let speed_clock = SpeedClock::new();
        speed_clock.set_speed(0.0, speed);
        let hand_on_redraw = |system: &System| {
            if screen_redraw.swap(false, Ordering::SeqCst) {
                system.event_queue().push(Event::Redraw);
            }
        };
        let mut wake_ticks = 0;
        // The last tick ended on a wake the host is to come back for: its frame is not over.
        let mut resumes = false;
        let start = clock.peek();
        for frame in 0..120 {
            log.frame.store(frame as u64 + 1, Ordering::SeqCst);
            let timer_floor_ms = timer_floors_ms[frame % timer_floors_ms.len()];
            let budget_end = clock.peek() + budget;
            if resumes {
                hand_on_redraw(&system);
            }
            system.tick_for(budget)?;
            let next_frame = (clock.peek() / frame_ms + 1) * frame_ms;
            loop {
                let wake = system.pacing().take().wake();
                resumes = wake.is_some();
                let Some((due, margin)) = wake.zip(margin) else {
                    hand_on_redraw(&system);
                    break;
                };
                let now = clock.peek();
                let delay = (speed_clock.wake_delay(now as f64 / speed, due, margin) * speed) as u64;
                let fire = now + delay.max((timer_floor_ms as f64 * speed).ceil() as u64);
                if fire >= next_frame {
                    break;
                }
                clock.set(fire);
                in_wake_tick.store(true, Ordering::SeqCst);
                let result = system.tick_for(budget);
                in_wake_tick.store(false, Ordering::SeqCst);
                result?;
                wake_ticks += 1;
                // A wake tick is the rest of its frame's tick, whatever budget it is handed: it ends
                // where that tick's budget did (`Executor::resume_tick`), a clock read or two past.
                let end = clock.peek();
                assert!(
                    end <= budget_end + 1,
                    "{guest:?} at {speed}x: a wake tick ran to {end}, {} guest ms past its frame tick's budget",
                    end - budget_end
                );
            }
            clock.set(next_frame);
        }
        Ok(HostRun {
            fires: fires.load(Ordering::SeqCst),
            beats: beats.load(Ordering::SeqCst),
            redraws: redraws.load(Ordering::SeqCst),
            redraws_in_a_frame: log.most_in_a_frame.load(Ordering::SeqCst),
            redraw_frames_late: log.most_frames_late.load(Ordering::SeqCst),
            callbacks: CALLBACK_RUNS.with(|runs| runs.get()),
            wake_ticks,
            ticks: system.pacing().ticks(),
            elapsed: clock.peek() - start,
        })
    }

    // `on_a_host` at 1x for a WIPI timer, as (fires, wake ticks, host frames counted); with
    // `beat_ms`, a second thread sleeps on that beat beside it.
    fn timer_fires_on_a_host(
        after_ms: u64,
        pace_after: Option<u64>,
        beat_ms: Option<u64>,
        margin: Option<u64>,
        timer_floor_ms: u64,
    ) -> Result<(u64, u64, u64)> {
        let guest = Guest {
            timer: Some((after_ms, pace_after)),
            beat_ms,
            ..Guest::default()
        };
        let run = on_a_host(guest, 1.0, margin, timer_floor_ms)?;
        Ok((run.fires, run.wake_ticks, run.ticks))
    }

    #[test]
    fn a_host_that_wakes_the_guest_still_fires_a_timer_once_per_host_frame() -> Result<()> {
        // A host that comes back for a guest's wake between frames (`Platform::wake_margin_ms`) hands
        // the tick's wait back; it must not hand a timer more fires than the host frames allow. When
        // it came back for every due time the event loop reported and counted each visit as a host
        // frame, MC_knlSetTimer(1) ran 58 -> 294~1000 times a second and a 10ms timer 58 -> 94: the
        // speeds `a_1ms_guest_timer_is_left_to_the_host_frame` and `MC_knlSetTimer`'s share of fires
        // per tick are there to prevent (docs/report/0556 §8).
        for timer_floor_ms in [1, 4] {
            let host = |after_ms, pace_after, beat_ms| -> Result<_> {
                let off = timer_fires_on_a_host(after_ms, pace_after, beat_ms, None, 0)?;
                let on = timer_fires_on_a_host(after_ms, pace_after, beat_ms, Some(3), timer_floor_ms)?;
                // A wake tick is not a host frame: a timer's share of fires goes by the tick's number
                // (`timer_fires_per_tick` — at 1.5x by whether it is odd), so the count must not move.
                assert_eq!((off.2, on.2), (120, 120), "host frames counted, {} of them wake ticks", on.1);
                Ok((off.0, on.0, on.1))
            };
            // KTF 49ade89578c5's MC_knlSetTimer(1), and a 10ms timer re-armed in the tick it fired in:
            // both left to the host frame, so there is nothing to come back for.
            let (off, on, wake_ticks) = host(1, None, None)?;
            assert_eq!((on, wake_ticks), (off, 0), "MC_knlSetTimer(1), timers no sooner than {timer_floor_ms}ms");
            let (off, on, wake_ticks) = host(10, Some(1), None)?;
            assert_eq!((on, wake_ticks), (off, 0), "a 10ms timer, timers no sooner than {timer_floor_ms}ms");
            // A 25ms timer does keep a tick alive: the host comes back for it, to the same fires.
            let (off, on, wake_ticks) = host(25, Some(0), None)?;
            assert!(wake_ticks > 0, "the host never came back for a 25ms timer");
            assert_eq!(on, off, "a 25ms timer, timers no sooner than {timer_floor_ms}ms");
            // The 10ms timer beside a thread on a 40ms beat, which the host does come back for: a wake
            // tick is the rest of its frame's tick, not a frame that owes the timer another fire.
            let (off, on, wake_ticks) = host(10, Some(1), Some(40))?;
            assert!(wake_ticks > 0, "the host never came back for the 40ms beat");
            assert_eq!(on, off, "a 10ms timer beside a 40ms beat, timers no sooner than {timer_floor_ms}ms");
        }
        Ok(())
    }

    #[test]
    fn a_poll_beside_a_paced_wake_runs_as_often_on_a_host_that_wakes_the_guest() -> Result<()> {
        // A tick that waits for a paced wake runs every poll beside it, a millisecond apart, for as
        // long as it waits: a 1ms WIPI timer beside a thread on a 30ms beat fires 567 times in 120
        // frames, not the 119 it fires alone. A host that takes that wait over
        // (`Platform::wake_margin_ms`) and came back only for the wake ran those polls 8-50% less
        // often — the game slower for having opted in. So a tick hands over only a wake no poll has
        // work before (`System::poll_until`), and none nearer than a host timer can fire
        // (`speed_clock::max_wait`): docs/report/0556 §9.
        let timer = |after_ms, pace_after, beat_ms| Guest {
            timer: Some((after_ms, pace_after)),
            beat_ms,
            ..Guest::default()
        };
        let guests = [
            // MC_knlSetTimer(1), and a 10ms timer re-armed in the tick it fired in, beside a beat.
            timer(1, None, Some(30)),
            timer(1, None, Some(40)),
            timer(1, None, Some(60)),
            timer(10, Some(1), Some(30)),
            timer(10, Some(1), Some(40)),
            // A thread in a `Thread.sleep(1)` loop beside one: the same poll, with no event loop.
            Guest {
                beat_ms: Some(30),
                poller: true,
                ..Guest::default()
            },
            // A `callSerially` chain beside one: the event loop polls toward the end of its 16ms
            // slice, where the next run is. Left out of what the tick is told, a 40ms beat's host ran
            // the chain 164 -> 153 times at 2x.
            Guest {
                beat_ms: Some(40),
                call_serially: true,
                ..Guest::default()
            },
            // A 25ms timer that paces its own tick: the host does come back for this one, and at 2x
            // most frames — where a wake tick with a budget of its own runs past the frame's
            // (`on_a_host` asserts it does not).
            timer(25, Some(0), None),
        ];
        let mut wake_ticks = 0;
        for speed in [1.0, 1.5, 2.0] {
            for guest in guests {
                let off = on_a_host(guest, speed, None, 0)?;
                for timer_floor_ms in [1, 4] {
                    let on = on_a_host(guest, speed, Some(3), timer_floor_ms)?;
                    wake_ticks += on.wake_ticks;
                    assert_eq!(
                        (on.fires, on.callbacks, on.elapsed, on.ticks),
                        (off.fires, off.callbacks, off.elapsed, off.ticks),
                        "{guest:?} at {speed}x, timers no sooner than {timer_floor_ms}ms: (fires, callSerially runs, guest ms, host frames) with {} wake ticks",
                        on.wake_ticks
                    );
                }
            }
        }
        // And it is not that the host never comes back: the paced timer and the beats still hand over.
        assert!(wake_ticks > 300, "{wake_ticks}");
        Ok(())
    }

    #[test]
    fn a_repaint_reaches_the_guest_once_per_host_frame_on_a_host_that_wakes_the_guest() -> Result<()> {
        // A repaint goes on to the event loop when the host frame's tick is over — one a frame, though
        // a thread that repaints and sleeps 10ms asks three times in a 2x frame. A host that takes the
        // tick's wait over (`Platform::wake_margin_ms`) ends that tick two or three times a frame; handed
        // on at each end, the same thread was painted on every beat (119 -> 359 Redraws in 120 frames),
        // and a game that moves in `paint()` ran that much faster for its host having opted in
        // (docs/report/0556 §10). It goes on only from a tick the host does not come back for.
        let beat = |beat_ms| Guest {
            beat_ms: Some(beat_ms),
            repaint: Some(Repaint::Engine),
            ..Guest::default()
        };
        let guests = [
            // `Display.repaint` then `Thread.sleep`, on a beat shorter than a tick's budget.
            beat(10),
            beat(20),
            // WIPI-C `MC_grpRepaint` from a timer that fires twice in a 2x frame: the host's half of
            // the same rule (`on_a_host`, as `wie_featurephone`'s `advance`).
            Guest {
                timer: Some((12, Some(0))),
                repaint: Some(Repaint::Screen),
                ..Guest::default()
            },
        ];
        let (mut wake_ticks, mut unpainted, mut some_wake_ticks) = (0, 0, 0);
        for speed in [1.0, 1.5, 2.0] {
            for guest in guests {
                let off = on_a_host(guest, speed, None, 0)?;
                assert!(off.redraws > 90, "{guest:?} at {speed}x: {} Redraws", off.redraws);
                unpainted += off.beats - off.redraws;
                for timer_floor_ms in [1, 4] {
                    let on = on_a_host(guest, speed, Some(3), timer_floor_ms)?;
                    wake_ticks += on.wake_ticks;
                    assert_eq!(
                        (on.redraws, on.beats, on.elapsed, on.ticks),
                        (off.redraws, off.beats, off.elapsed, off.ticks),
                        "{guest:?} at {speed}x, timers no sooner than {timer_floor_ms}ms: (Redraws, beats, guest ms, host frames) with {} wake ticks",
                        on.wake_ticks
                    );
                }
                // A host whose timer always loses to the next frame never runs the wake tick that would
                // end the frame's tick. The frame is over all the same, and the next one starts by
                // handing the repaint on: one for every beat it gets to, at most one a frame. Handed on
                // only where a tick ends, the 10ms beat — whose every tick ends on a wake — was never
                // painted at all.
                let late = on_a_host(guest, speed, Some(3), 1_000)?;
                assert_eq!(late.wake_ticks, 0);
                assert!(
                    late.redraws + 1 >= late.beats.min(off.redraws),
                    "{guest:?} at {speed}x, a host that never comes back: {} Redraws for {} beats, {} without a margin",
                    late.redraws,
                    late.beats,
                    off.redraws
                );
                // Between the two, a host whose timer wins two frames and loses the third: a repaint
                // held back for a wake tick that then never runs is where one could be left behind,
                // or handed on beside the next. Still at most one Redraw a frame, each in the frame
                // after its repaint at the latest — as without a margin.
                let some = on_a_host_with_timers(guest, speed, Some(3), &[1, 1, 30])?;
                some_wake_ticks += some.wake_ticks;
                assert_eq!(
                    (
                        off.redraws_in_a_frame,
                        off.redraw_frames_late,
                        some.redraws_in_a_frame,
                        some.redraw_frames_late
                    ),
                    (1, 1, 1, 1),
                    "{guest:?} at {speed}x: (most Redraws in a frame, most frames from a repaint to its Redraw) without a margin, \
                     and on a host that comes back two frames in three ({} wake ticks)",
                    some.wake_ticks
                );
            }
        }
        // And these are the shapes it is about: more beats than frames, and a host that does come back.
        assert!(
            unpainted > 500 && wake_ticks > 1_000 && some_wake_ticks > 500,
            "{unpainted} beats unpainted, {wake_ticks} wake ticks, {some_wake_ticks} on the host that comes back in some frames"
        );
        Ok(())
    }

    #[test]
    fn a_repaint_waits_for_the_host_s_wake_tick_only_on_a_host_that_comes_back() -> Result<()> {
        // A guest that repaints and sleeps 10ms, on a clock that stands still: the tick ends on a wake
        // inside its budget — to a host with a margin, the tick it comes back for, so the repaint
        // waits for that wake tick. A host without one comes back for nothing: there every tick end
        // hands the repaint on, as it always has.
        for (margin, handed_on) in [(None, true), (Some(3), false)] {
            let platform = TestPlatform::with_clock(TestClock::new());
            let platform = margin.into_iter().fold(platform, TestPlatform::with_wake_margin);
            let mut system = System::new(Box::new(platform), "", "", DefaultTaskRunner);
            let system_task = system.clone();
            system.spawn(async move || {
                system_task.request_redraw();
                system_task.sleep(10).await;
                Ok(())
            });
            system.tick()?;
            assert_eq!(system.pacing().wake().map(|x| x.raw()), margin.map(|_| 10), "margin {margin:?}");
            let redraw = matches!(system.event_queue().pop(), Some(Event::Redraw));
            assert_eq!(redraw, handed_on, "margin {margin:?}");
        }
        Ok(())
    }

    #[test]
    fn recurring_callback_keeps_the_slice_when_a_timer_is_pending() -> Result<()> {
        // IdleCallback asserts ≥16ms between its runs; waking early for the timer must not
        // re-run recurring callSerially work at the timer's cadence.
        let callback_proto = WieJavaClassProto {
            name: "net/wie/IdleCallback",
            parent_class: Some("java/lang/Object"),
            interfaces: vec!["java/lang/Runnable"],
            methods: vec![JavaMethodProto::new("run", "()V", IdleCallback::run, MethodAccessFlags::PUBLIC)],
            fields: vec![
                JavaFieldProto::new("count", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("lastRun", "J", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        };
        let (_, at_5ms) = get_next_event_with_timer_due_at_5ms(4, vec![callback_proto], Some("net/wie/IdleCallback"))?;
        assert!(!at_5ms, "with recurring callbacks pending the timer waits for the 16ms slice");
        Ok(())
    }

    #[test]
    fn event_queued_mid_wait_returns_without_waiting_for_the_whole_slice() -> Result<()> {
        let clock = TestClock::new();
        let mut system = System::new(Box::new(TestPlatform::with_clock(clock.clone())), "", "", DefaultTaskRunner);
        let completed = Arc::new(AtomicBool::new(false));
        let completed_task = completed.clone();
        let system_task = system.clone();
        let clock_task = clock.clone();
        system.spawn(async move || {
            let jvm = JvmSupport::new_jvm(&system_task, None, Box::new([get_protos().into()]), &[], RustJavaJvmImplementation).await?;
            let queue = jvm
                .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
                .await
                .unwrap();
            let event = jvm.instantiate_array("I", 4).await.unwrap();
            let _: () = jvm
                .invoke_virtual(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event,))
                .await
                .unwrap();
            completed_task.store(true, Ordering::SeqCst);
            clock_task.advance(100);
            Ok(())
        });

        system.tick()?;
        assert!(!completed.load(Ordering::SeqCst), "the queue is empty, so the call must wait");
        clock.set(3);
        system.event_queue().push(Event::Redraw);
        clock.set(4);
        system.tick()?;
        assert!(
            completed.load(Ordering::SeqCst),
            "an event queued mid-wait must be returned within a millisecond"
        );
        Ok(())
    }

    #[test]
    fn a_repaint_handed_over_in_the_step_of_a_paint_is_looked_at_in_the_next_step() -> Result<()> {
        // A game thread that spins on Thread.yield until its paint lands, with no sleep: the event
        // thread paints for 3ms of a step and goes back to an empty queue, and the game thread —
        // polled after it in that same step — hands over the next repaint. The 1ms poll counts from
        // that step's start, so it is already up; counted from the paint's end (`now + 1`) the event
        // thread sat out another millisecond of game-thread spinning per frame (SKT b42e4242866b
        // 387 -> 329 paints/s after #603).
        let clock = TestClock::new();
        let mut system = System::new(Box::new(TestPlatform::with_clock(clock.clone())), "", "", DefaultTaskRunner);
        let painted = Arc::new(AtomicBool::new(false));
        let completed = Arc::new(AtomicBool::new(false));
        let (system_task, clock_task, painted_task, completed_task) = (system.clone(), clock.clone(), painted.clone(), completed.clone());
        system.spawn(async move || {
            let jvm = JvmSupport::new_jvm(&system_task, None, Box::new([get_protos().into()]), &[], RustJavaJvmImplementation).await?;
            let queue = jvm
                .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
                .await
                .unwrap();
            let event = jvm.instantiate_array("I", 4).await.unwrap();
            // The paint: this step started 3ms ago by the time the thread is back at its queue.
            clock_task.advance(3);
            painted_task.store(true, Ordering::SeqCst);
            let _: () = jvm
                .invoke_virtual(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event,))
                .await
                .unwrap();
            completed_task.store(true, Ordering::SeqCst);
            // Let the enclosing tick finish after this task completes.
            clock_task.advance(100);
            Ok(())
        });
        let (system_game, painted_game) = (system.clone(), painted.clone());
        system.spawn(async move || {
            while !painted_game.load(Ordering::SeqCst) {
                YieldFuture::new().await;
            }
            system_game.event_queue().push(Event::Redraw);
            Ok(())
        });

        system.tick()?;
        assert_eq!(clock.peek(), 103, "the wait must not outlast the step after the paint's");
        assert!(completed.load(Ordering::SeqCst));
        Ok(())
    }

    struct RecurringCallback;

    impl RecurringCallback {
        async fn paint(
            jvm: &Jvm,
            _context: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            graphics: ClassInstanceRef<Graphics>,
        ) -> JvmResult<()> {
            let paint_count: i32 = jvm.get_field(&this, "net/wie/RecurringCallback", "paintCount", "I").await?;
            jvm.put_field(&mut this, "net/wie/RecurringCallback", "paintCount", "I", paint_count + 1)
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0x22aa44,))
                .await?;
            if jvm.get_field::<bool>(&this, "net/wie/RecurringCallback", "repaintOnPaint", "Z").await? {
                let _: () = jvm.invoke_virtual(&this, "javax/microedition/lcdui/Canvas", "repaint", "()V", ()).await?;
            }
            jvm.invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 1, 1))
                .await
        }

        async fn run(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
            let count: i32 = jvm.get_field(&this, "net/wie/RecurringCallback", "count", "I").await?;
            let display: ClassInstanceRef<Display> = jvm
                .get_field(
                    &this,
                    "javax/microedition/lcdui/Displayable",
                    "currentDisplay",
                    "Ljavax/microedition/lcdui/Display;",
                )
                .await?;
            let mut graphics: ClassInstanceRef<Graphics> = jvm
                .invoke_virtual(
                    &display,
                    "javax/microedition/lcdui/Display",
                    "getScreenGraphics",
                    "()Ljavax/microedition/lcdui/Graphics;",
                    (),
                )
                .await?;
            let image = Graphics::image(jvm, &mut graphics).await?;
            let pixel = Image::image(jvm, &image).await?.get_pixel(0, 0);
            assert_eq!(
                (pixel.r, pixel.g, pixel.b),
                (0x22, 0xaa, 0x44),
                "pending Canvas paint must finish before run"
            );
            jvm.put_field(&mut this, "net/wie/RecurringCallback", "count", "I", count + 1).await?;
            if jvm.get_field::<bool>(&this, "net/wie/RecurringCallback", "repeat", "Z").await? {
                let _: () = jvm.invoke_virtual(&this, "javax/microedition/lcdui/Canvas", "repaint", "()V", ()).await?;
                let queue: ClassInstanceRef<EventQueue> = jvm
                    .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&queue, "net/wie/EventQueue", "callSerially", "(Ljava/lang/Runnable;)V", (this,))
                    .await?;
            }
            Ok(())
        }
    }

    #[test]
    fn input_precedes_recurring_callbacks_and_pending_paint() -> Result<()> {
        let callback_proto = WieJavaClassProto {
            name: "net/wie/RecurringCallback",
            parent_class: Some("javax/microedition/lcdui/Canvas"),
            interfaces: vec!["java/lang/Runnable"],
            methods: vec![
                JavaMethodProto::new("run", "()V", RecurringCallback::run, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "paint",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    RecurringCallback::paint,
                    MethodAccessFlags::PROTECTED,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("count", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("paintCount", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("repeat", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("repaintOnPaint", "Z", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        };
        let midlet_proto = WieJavaClassProto {
            name: "net/wie/QueueTestMidlet",
            parent_class: Some("javax/microedition/midlet/MIDlet"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        };
        run_jvm_test_with_system(
            Box::new([get_protos().into(), Box::new([callback_proto, midlet_proto])]),
            Box::new(TestPlatform::new()),
            |jvm, system| async move {
                let queue: ClassInstanceRef<EventQueue> = jvm
                    .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
                    .await?;
                let midlet: ClassInstanceRef<MIDlet> = jvm.instantiate_class("net/wie/QueueTestMidlet").await?.into();
                let _: () = jvm
                    .invoke_special(&midlet, "javax/microedition/midlet/MIDlet", "<init>", "()V", ())
                    .await?;
                let display = MIDlet::display(&jvm, &midlet).await?;
                let mut callback: ClassInstanceRef<RecurringCallback> = jvm.instantiate_class("net/wie/RecurringCallback").await?.into();
                jvm.put_field(&mut callback, "net/wie/RecurringCallback", "repeat", "Z", true).await?;
                let _: () = jvm
                    .invoke_special(&callback, "javax/microedition/lcdui/Canvas", "<init>", "()V", ())
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        &display,
                        "javax/microedition/lcdui/Display",
                        "setCurrent",
                        "(Ljavax/microedition/lcdui/Displayable;)V",
                        (callback.clone(),),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&callback, "javax/microedition/lcdui/Canvas", "repaint", "()V", ())
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        &display,
                        "javax/microedition/lcdui/Display",
                        "callSerially",
                        "(Ljava/lang/Runnable;)V",
                        (callback.clone(),),
                    )
                    .await?;
                assert!(system.event_queue().pop().is_none(), "callSerially must not schedule host events");
                let _: () = jvm
                    .invoke_virtual(
                        &display,
                        "javax/microedition/lcdui/Display",
                        "callSerially",
                        "(Ljava/lang/Runnable;)V",
                        (callback.clone(),),
                    )
                    .await?;
                system.event_queue().push(Event::Redraw);
                system.event_queue().push(Event::Keydown(KeyCode::NUM1));
                system.event_queue().push(Event::Keyup(KeyCode::NUM1));
                let event: ClassInstanceRef<Array<i32>> = jvm.instantiate_array("I", 4).await?.into();
                let _: () = jvm
                    .invoke_virtual(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event.clone(),))
                    .await?;
                assert_eq!(jvm.load_array::<i32>(&event, 0, 1).await?, [EventQueueEvent::KeyEvent as i32]);
                assert_eq!(jvm.get_field::<i32>(&callback, "net/wie/RecurringCallback", "count", "I").await?, 0);
                let _: () = jvm
                    .invoke_virtual(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event.clone(),))
                    .await?;
                assert_eq!(jvm.load_array::<i32>(&event, 0, 4).await?, [EventQueueEvent::KeyEvent as i32, 2, 49, 0]);
                assert_eq!(jvm.get_field::<i32>(&callback, "net/wie/RecurringCallback", "count", "I").await?, 0);

                system.event_queue().push(Event::Keydown(KeyCode::NUM2));
                let _: () = jvm
                    .invoke_virtual(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event.clone(),))
                    .await?;
                assert_eq!(jvm.load_array::<i32>(&event, 0, 4).await?, [EventQueueEvent::KeyEvent as i32, 1, 50, 0]);
                assert_eq!(jvm.get_field::<i32>(&callback, "net/wie/RecurringCallback", "count", "I").await?, 0);

                system.event_queue().push(Event::Keydown(KeyCode::NUM3));
                system.event_queue().push(Event::Keyup(KeyCode::NUM3));
                for index in 0..10 {
                    // New input retains priority over pending redraw and callback work.
                    system.event_queue().push(Event::Keyrepeat(KeyCode::NUM3));
                    let _: () = jvm
                        .invoke_virtual(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event.clone(),))
                        .await?;
                    let event_type = match index {
                        0 => 1,
                        1 => 2,
                        _ => 3,
                    };
                    assert_eq!(
                        jvm.load_array::<i32>(&event, 0, 4).await?,
                        [EventQueueEvent::KeyEvent as i32, event_type, 51, 0]
                    );
                    assert_eq!(jvm.get_field::<i32>(&callback, "net/wie/RecurringCallback", "count", "I").await?, 0);
                }
                for _ in 0..2 {
                    let _: () = jvm
                        .invoke_virtual(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event.clone(),))
                        .await?;
                    assert_eq!(jvm.load_array::<i32>(&event, 0, 4).await?, [EventQueueEvent::KeyEvent as i32, 3, 51, 0]);
                    assert_eq!(jvm.get_field::<i32>(&callback, "net/wie/RecurringCallback", "count", "I").await?, 0);
                }
                for count in 1..=2 {
                    jvm.put_field(&mut callback, "net/wie/RecurringCallback", "repaintOnPaint", "Z", count == 2)
                        .await?;
                    system.event_queue().push(Event::Redraw);
                    let _: () = jvm
                        .invoke_virtual(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event.clone(),))
                        .await?;
                    assert_eq!(jvm.load_array::<i32>(&event, 0, 1).await?, [EventQueueEvent::RepaintEvent as i32]);
                    let _: () = jvm
                        .invoke_virtual(&queue, "net/wie/EventQueue", "dispatchEvent", "([I)V", (event.clone(),))
                        .await?;
                    assert_eq!(
                        jvm.get_field::<i32>(&callback, "net/wie/RecurringCallback", "count", "I").await?,
                        count * 2
                    );
                    assert_eq!(
                        jvm.get_field::<i32>(&callback, "net/wie/RecurringCallback", "paintCount", "I").await?,
                        count
                    );
                    assert!(
                        jvm.get_field::<bool>(&display, "javax/microedition/lcdui/Display", "repaintPending", "Z")
                            .await?
                    );
                }
                jvm.put_field(&mut callback, "net/wie/RecurringCallback", "repaintOnPaint", "Z", false)
                    .await?;
                jvm.put_field(&mut callback, "net/wie/RecurringCallback", "repeat", "Z", false).await?;
                for paint_count in 3..=4 {
                    // First finish the callbacks, then repaint with no pending request or callback.
                    system.event_queue().push(Event::Redraw);
                    let _: () = jvm
                        .invoke_virtual(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event.clone(),))
                        .await?;
                    let _: () = jvm
                        .invoke_virtual(&queue, "net/wie/EventQueue", "dispatchEvent", "([I)V", (event.clone(),))
                        .await?;
                    assert_eq!(jvm.get_field::<i32>(&callback, "net/wie/RecurringCallback", "count", "I").await?, 6);
                    assert_eq!(
                        jvm.get_field::<i32>(&callback, "net/wie/RecurringCallback", "paintCount", "I").await?,
                        paint_count
                    );
                    assert!(
                        !jvm.get_field::<bool>(&display, "javax/microedition/lcdui/Display", "repaintPending", "Z")
                            .await?
                    );
                }
                Ok(())
            },
        )
    }
}
