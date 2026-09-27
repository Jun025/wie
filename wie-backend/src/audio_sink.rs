use alloc::{sync::Arc, vec::Vec};

pub type AudioHandle = u32;

#[derive(Clone, Debug, PartialEq)]
pub enum AudioCommand {
    Play {
        handle: AudioHandle,
        sequence: Arc<AudioSequence>,
        repeat: bool,
    },
    Stop {
        handle: AudioHandle,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct AudioSequence {
    pub duration: u64,
    pub events: Vec<TimedAudioEvent>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimedAudioEvent {
    pub time: u64,
    pub data: AudioEventData,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AudioEventData {
    Midi(Vec<u8>),
    Wave { channels: u8, sampling_rate: u32, samples: Vec<i16> },
}

pub trait AudioSink: Sync + Send {
    fn send(&self, command: AudioCommand);

    /// Output gain of `handle` (0.0 = silent, 1.0 = as authored), for the handle's current
    /// playback if one is sounding, otherwise for its next `Play`. `Audio` calls this right before
    /// every `Play` and again whenever the gain of a playing handle changes, so a sink never has to
    /// remember a gain past the next `Play`.
    ///
    /// A method rather than an `AudioCommand` variant so that sinks which ignore volume need no
    /// change: sinks match `AudioCommand` exhaustively, and a new variant breaks every one of them.
    fn set_gain(&self, _handle: AudioHandle, _gain: f32) {}
}
