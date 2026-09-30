use alloc::{
    boxed::Box,
    collections::{BTreeMap, VecDeque},
};
use core::pin::Pin;

use wie_util::Result;

use crate::Instant;

#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum KeyCode {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    OK,
    LEFT_SOFT_KEY,
    RIGHT_SOFT_KEY,
    CLEAR,
    CALL,
    HANGUP,
    VOLUME_UP,
    VOLUME_DOWN,

    NUM0,
    NUM1,
    NUM2,
    NUM3,
    NUM4,
    NUM5,
    NUM6,
    NUM7,
    NUM8,
    NUM9,
    HASH,
    STAR,
}

impl KeyCode {
    // TODO we can use libraries like strum
    pub fn parse(string: &str) -> KeyCode {
        match string {
            "UP" => KeyCode::UP,
            "DOWN" => KeyCode::DOWN,
            "LEFT" => KeyCode::LEFT,
            "RIGHT" => KeyCode::RIGHT,
            "OK" => KeyCode::OK,
            "0" => KeyCode::NUM0,
            "1" => KeyCode::NUM1,
            "2" => KeyCode::NUM2,
            "3" => KeyCode::NUM3,
            "4" => KeyCode::NUM4,
            "5" => KeyCode::NUM5,
            "6" => KeyCode::NUM6,
            "7" => KeyCode::NUM7,
            "8" => KeyCode::NUM8,
            "9" => KeyCode::NUM9,
            "#" => KeyCode::HASH,
            "*" => KeyCode::STAR,
            "CLR" => KeyCode::CLEAR,
            _ => unimplemented!("Unknown key: {string}"),
        }
    }
}

type TimerCallback = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;

pub enum Event {
    Redraw,
    Keydown(KeyCode),
    Keyup(KeyCode),
    Keyrepeat(KeyCode),
    // `pace_from`: see `Event::guest_timer`
    Timer { due: Instant, pace_from: u64, callback: TimerCallback },
    Notify { r#type: i32, param1: i32, param2: i32 }, // wipi notifyEvent
}

impl Event {
    pub fn timer<F, Fut>(due: Instant, callback: F) -> Self
    where
        F: FnOnce() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        Self::guest_timer(due, 0, callback)
    }

    /// A guest timer. `pace_from`: the first host tick (`Pacing::ticks`) that may be kept alive
    /// until it is due; before that tick the host's next frame paces it, and `u64::MAX` never.
    pub fn guest_timer<F, Fut>(due: Instant, pace_from: u64, callback: F) -> Self
    where
        F: FnOnce() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        Event::Timer {
            due,
            pace_from,
            callback: Box::new(move || Box::pin(callback())),
        }
    }
}

#[derive(Default)]
pub struct EventQueue {
    input_events: VecDeque<Event>,
    events: VecDeque<Event>,
    // guest timer, by its address
    timers: BTreeMap<u32, GuestTimer>,
}

#[derive(Default)]
struct GuestTimer {
    armings: u64,
    cancelled_through: u64,
    fired_in_tick: Option<u64>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, event: Event) {
        if matches!(event, Event::Keydown(_) | Event::Keyup(_) | Event::Keyrepeat(_)) {
            self.input_events.push_back(event);
            return;
        }
        if matches!(event, Event::Redraw) && self.events.iter().any(|event| matches!(event, Event::Redraw)) {
            return;
        }
        self.events.push_back(event);
    }

    pub fn is_empty(&self) -> bool {
        self.input_events.is_empty() && self.events.is_empty()
    }

    /// Numbers an arming of a guest timer (WIPI `MC_knlSetTimer`), for `is_timer_armed` when it
    /// falls due. One still pending is left alone — two Sets give two callbacks, as they always did.
    pub fn arm_timer(&mut self, timer: u32) -> u64 {
        let timer = self.timers.entry(timer).or_default();
        timer.armings += 1;
        timer.armings
    }

    /// WIPI `MC_knlUnsetTimer`: every arming so far stops counting.
    pub fn cancel_timer(&mut self, timer: u32) {
        let timer = self.timers.entry(timer).or_default();
        timer.cancelled_through = timer.armings;
    }

    pub fn is_timer_armed(&self, timer: u32, arming: u64) -> bool {
        arming > self.timers.get(&timer).map_or(0, |x| x.cancelled_through)
    }

    pub fn timer_fired(&mut self, timer: u32, tick: u64) {
        self.timers.entry(timer).or_default().fired_in_tick = Some(tick);
    }

    pub fn timer_fired_in(&self, timer: u32, tick: u64) -> bool {
        self.timers.get(&timer).and_then(|x| x.fired_in_tick) == Some(tick)
    }

    /// Keyboard input takes priority; events at the same priority remain FIFO.
    pub fn pop(&mut self) -> Option<Event> {
        self.input_events.pop_front().or_else(|| self.events.pop_front())
    }
}

#[cfg(test)]
mod tests {
    use crate::Instant;

    use super::{Event, EventQueue, KeyCode};

    #[test]
    fn prioritizes_input_and_coalesces_pending_redraws() {
        let mut queue = EventQueue::new();
        queue.push(Event::Keydown(KeyCode::DOWN));
        queue.push(Event::Redraw);
        queue.push(Event::Keyrepeat(KeyCode::DOWN));
        for _ in 0..100 {
            queue.push(Event::Redraw);
        }
        queue.push(Event::Keyup(KeyCode::DOWN));

        assert!(matches!(queue.pop(), Some(Event::Keydown(KeyCode::DOWN))));
        assert!(matches!(queue.pop(), Some(Event::Keyrepeat(KeyCode::DOWN))));
        assert!(matches!(queue.pop(), Some(Event::Keyup(KeyCode::DOWN))));
        assert!(matches!(queue.pop(), Some(Event::Redraw)));
        // A repaint requested while painting still needs another delivery.
        queue.push(Event::Redraw);
        assert!(matches!(queue.pop(), Some(Event::Redraw)));
        assert!(queue.pop().is_none());
    }

    #[test]
    fn new_input_precedes_timers_and_notifications_without_reordering_them() {
        let mut queue = EventQueue::new();
        queue.push(Event::timer(Instant::from_epoch_millis(10), || async { Ok(()) }));
        queue.push(Event::Notify {
            r#type: 1,
            param1: 2,
            param2: 3,
        });
        queue.push(Event::Keydown(KeyCode::OK));
        assert!(matches!(queue.pop(), Some(Event::Keydown(KeyCode::OK))));
        queue.push(Event::Keyup(KeyCode::OK));
        assert!(matches!(queue.pop(), Some(Event::Keyup(KeyCode::OK))));
        assert!(matches!(queue.pop(), Some(Event::Timer { due, .. }) if due.raw() == 10));
        assert!(matches!(
            queue.pop(),
            Some(Event::Notify {
                r#type: 1,
                param1: 2,
                param2: 3
            })
        ));
        assert!(queue.pop().is_none());
    }
}
