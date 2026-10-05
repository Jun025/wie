use alloc::{boxed::Box, collections::BTreeMap, collections::BTreeSet, sync::Arc, vec, vec::Vec};

use smaf_player::{SmafEvent, parse_smaf};

use crate::{AudioCommand, AudioEventData, AudioHandle, AudioSequence, AudioSink, TimedAudioEvent};

#[derive(Debug)]
pub enum AudioError {
    InvalidHandle,
}

pub struct Audio {
    sink: Box<dyn AudioSink>,
    files: BTreeMap<AudioHandle, Arc<AudioSequence>>,
    playing: BTreeSet<AudioHandle>,
    last_audio_handle: AudioHandle,
    /// Volume the game set on a handle (0.0..=1.0); absent = 1.0.
    volumes: BTreeMap<AudioHandle, f32>,
    /// Volume the game set for all its sound (0.0..=1.0) — multiplies every handle's.
    master_volume: f32,
}

impl Audio {
    pub fn new(sink: Box<dyn AudioSink>) -> Self {
        Self {
            sink,
            files: BTreeMap::new(),
            playing: BTreeSet::new(),
            last_audio_handle: 0,
            volumes: BTreeMap::new(),
            master_volume: 1.0,
        }
    }

    pub fn load_smaf(&mut self, data: &[u8]) -> Result<AudioHandle, AudioError> {
        let audio_handle = self.last_audio_handle;
        let sequence = Arc::new(convert_smaf_events(parse_smaf_in(data)));

        self.last_audio_handle += 1;
        self.files.insert(audio_handle, sequence);

        Ok(audio_handle)
    }

    pub fn play(&mut self, audio_handle: AudioHandle, repeat: bool) -> Result<(), AudioError> {
        let sequence = self.files.get(&audio_handle).cloned().ok_or(AudioError::InvalidHandle)?;

        self.stop(audio_handle);
        self.playing.insert(audio_handle);
        self.sink.set_gain(audio_handle, self.gain(audio_handle));
        self.sink.send(AudioCommand::Play {
            handle: audio_handle,
            sequence,
            repeat,
        });

        Ok(())
    }

    /// Milliseconds from the start of the sequence to its last event.
    pub fn duration(&self, audio_handle: AudioHandle) -> Option<u64> {
        self.files.get(&audio_handle).map(|sequence| sequence.duration)
    }

    pub fn stop(&mut self, audio_handle: AudioHandle) {
        if self.playing.remove(&audio_handle) {
            self.sink.send(AudioCommand::Stop { handle: audio_handle });
        }
    }

    pub fn close(&mut self, audio_handle: AudioHandle) -> Result<(), AudioError> {
        self.stop(audio_handle);

        self.volumes.remove(&audio_handle);
        if self.files.remove(&audio_handle).is_none() {
            return Err(AudioError::InvalidHandle);
        }

        Ok(())
    }

    /// The game's volume for one handle, `0.0..=1.0`. Takes effect at once if it is playing.
    pub fn set_volume(&mut self, audio_handle: AudioHandle, volume: f32) -> Result<(), AudioError> {
        if !self.files.contains_key(&audio_handle) {
            return Err(AudioError::InvalidHandle);
        }

        self.volumes.insert(audio_handle, volume.clamp(0.0, 1.0));
        if self.playing.contains(&audio_handle) {
            self.sink.set_gain(audio_handle, self.gain(audio_handle));
        }

        Ok(())
    }

    pub fn volume(&self, audio_handle: AudioHandle) -> f32 {
        self.volumes.get(&audio_handle).copied().unwrap_or(1.0)
    }

    /// The game's volume for all its sound, `0.0..=1.0`, on top of each handle's own.
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
        for &audio_handle in &self.playing {
            self.sink.set_gain(audio_handle, self.gain(audio_handle));
        }
    }

    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    fn gain(&self, audio_handle: AudioHandle) -> f32 {
        self.master_volume * self.volume(audio_handle)
    }
}

// A KTF title hands `MC_mdaClipPutData` a 13,000-byte buffer holding a 10,201-byte SMAF file;
// `Smaf::parse` reads chunks up to the end of the slice and needs exactly the CRC after the last
// one, so the padding failed the whole parse and every play was an empty sequence. When the whole
// buffer yields nothing, parse it again cut at the length the `MMMD` header declares. Only then:
// a header that misstates its length (0 in a committed test) still parses whole, as it always did.
// Public because MIDP's `SmafPlayer` measures the clip length off the same events.
// The declared length is a u32 straight from the file: `saturating_add`, or a broken header near
// u32::MAX overflows (a debug panic on wasm32, where usize is 32 bits).
pub fn parse_smaf_in(data: &[u8]) -> Vec<(usize, SmafEvent)> {
    let events = parse_smaf(data);
    match data {
        [b'M', b'M', b'M', b'D', a, b, c, d, ..] if events.is_empty() => {
            parse_smaf(&data[..data.len().min(u32::from_be_bytes([*a, *b, *c, *d]).saturating_add(8) as usize)])
        }
        _ => events,
    }
}

fn convert_smaf_events(events: Vec<(usize, SmafEvent)>) -> AudioSequence {
    let duration = events.iter().map(|(time, _)| *time as u64).max().unwrap_or(0);
    let events = events
        .into_iter()
        .filter_map(|(time, event)| {
            let data = match event {
                SmafEvent::Wave {
                    channel,
                    sampling_rate,
                    data,
                } => AudioEventData::Wave {
                    channels: channel,
                    sampling_rate,
                    samples: data,
                },
                SmafEvent::MidiNoteOn { channel, note, velocity } => AudioEventData::Midi(vec![0x90 | channel, note, velocity]),
                SmafEvent::MidiNoteOff { channel, note, velocity } => AudioEventData::Midi(vec![0x80 | channel, note, velocity]),
                SmafEvent::MidiProgramChange { channel, program } => AudioEventData::Midi(vec![0xc0 | channel, program]),
                SmafEvent::MidiControlChange { channel, control, value } => AudioEventData::Midi(vec![0xb0 | channel, control, value]),
                SmafEvent::MidiPitchBend { channel, value } => {
                    AudioEventData::Midi(vec![0xe0 | channel, (value & 0x7f) as u8, ((value >> 7) & 0x7f) as u8])
                }
                SmafEvent::MidiSysEx(data) => AudioEventData::Midi(data),
                SmafEvent::End => return None,
            };

            Some(TimedAudioEvent { time: time as u64, data })
        })
        .collect();

    AudioSequence { duration, events }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use alloc::{boxed::Box, sync::Arc, vec, vec::Vec};
    use std::sync::Mutex;

    use smaf_player::SmafEvent;

    use super::{Audio, convert_smaf_events};
    use crate::{AudioCommand, AudioEventData, AudioSequence, AudioSink, TimedAudioEvent};

    struct RecordingSink(Arc<Mutex<Vec<AudioCommand>>>);

    impl AudioSink for RecordingSink {
        fn send(&self, command: AudioCommand) {
            self.0.lock().unwrap().push(command);
        }
    }

    #[test]
    fn converts_smaf_events_to_timed_transport() {
        let sequence = convert_smaf_events(vec![
            (5, SmafEvent::MidiProgramChange { channel: 2, program: 7 }),
            (10, SmafEvent::MidiPitchBend { channel: 3, value: 0x1234 }),
            (15, SmafEvent::End),
        ]);

        assert_eq!(
            sequence,
            AudioSequence {
                duration: 15,
                events: vec![
                    TimedAudioEvent {
                        time: 5,
                        data: AudioEventData::Midi(vec![0xc2, 7]),
                    },
                    TimedAudioEvent {
                        time: 10,
                        data: AudioEventData::Midi(vec![0xe3, 0x34, 0x24]),
                    },
                ],
            }
        );
    }

    // One note in a format-2 score track, then a buffer padded past the file. Padding is only fatal
    // for some lengths (here 6 or 7 mod 8 — the measured title's 2,799 is one of them).
    #[test]
    fn smaf_in_a_padded_buffer_still_plays_its_notes() {
        let seq = [0x00, 0x90, 0x3c, 0x40, 0x10, 0x00, 0xff, 0x2f, 0x00];
        let mut mtr = vec![0x02, 0x00, 0x02, 0x02];
        mtr.extend([0; 16]);
        mtr.extend(b"Mtsq");
        mtr.extend((seq.len() as u32).to_be_bytes());
        mtr.extend(seq);
        let mut file = b"MMMD".to_vec();
        file.extend((8 + mtr.len() as u32 + 2).to_be_bytes());
        file.extend(b"MTR\x05");
        file.extend((mtr.len() as u32).to_be_bytes());
        file.extend(mtr);
        file.extend([0, 0]);
        file.extend([0; 6]);

        let mut audio = Audio::new(Box::new(RecordingSink(Arc::new(Mutex::new(Vec::new())))));
        let handle = audio.load_smaf(&file).unwrap();

        assert!(!audio.files[&handle].events.is_empty());
    }

    // A broken file whose header declares a length near u32::MAX: no panic, just nothing to play.
    #[test]
    fn smaf_declaring_a_length_near_u32_max_loads_empty() {
        let mut file = b"MMMD".to_vec();
        file.extend(u32::MAX.to_be_bytes());
        file.extend(b"MTR\x05garbage");

        assert!(super::parse_smaf_in(&file).is_empty());
    }

    #[test]
    fn replay_stops_previous_playback_before_starting_again() {
        let commands = Arc::new(Mutex::new(Vec::new()));
        let mut audio = Audio::new(Box::new(RecordingSink(commands.clone())));
        let handle = audio.load_smaf(&[]).unwrap();

        audio.play(handle, false).unwrap();
        audio.play(handle, true).unwrap();

        let commands = commands.lock().unwrap();
        let AudioCommand::Play {
            sequence: first_sequence,
            repeat: false,
            ..
        } = &commands[0]
        else {
            panic!("expected initial play command");
        };
        assert_eq!(commands[1], AudioCommand::Stop { handle });
        let AudioCommand::Play {
            sequence: second_sequence,
            repeat: true,
            ..
        } = &commands[2]
        else {
            panic!("expected replay command");
        };
        assert!(Arc::ptr_eq(first_sequence, second_sequence));
    }

    #[test]
    fn close_stops_playback_and_removes_the_handle() {
        let commands = Arc::new(Mutex::new(Vec::new()));
        let mut audio = Audio::new(Box::new(RecordingSink(commands.clone())));
        let handle = audio.load_smaf(&[]).unwrap();

        audio.play(handle, false).unwrap();
        audio.close(handle).unwrap();

        assert_eq!(commands.lock().unwrap()[1], AudioCommand::Stop { handle });
        assert!(audio.play(handle, false).is_err());
    }
    struct GainSink(Arc<Mutex<Vec<(&'static str, u32, f32)>>>);

    impl AudioSink for GainSink {
        fn send(&self, command: AudioCommand) {
            let entry = match command {
                AudioCommand::Play { handle, .. } => ("play", handle, 0.0),
                AudioCommand::Stop { handle } => ("stop", handle, 0.0),
            };
            self.0.lock().unwrap().push(entry);
        }

        fn set_gain(&self, handle: u32, gain: f32) {
            self.0.lock().unwrap().push(("gain", handle, gain));
        }
    }

    // Every play is preceded by its gain, so a sink never keeps one past the next play; a change
    // reaches the sink only for a playing handle; master multiplies; close forgets the volume.
    #[test]
    fn volume_reaches_the_sink_before_every_play_and_while_playing() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut audio = Audio::new(Box::new(GainSink(log.clone())));
        let a = audio.load_smaf(&[]).unwrap();
        let b = audio.load_smaf(&[]).unwrap();

        audio.set_volume(a, 0.5).unwrap();
        audio.play(a, true).unwrap();
        audio.set_master_volume(0.4);
        audio.set_volume(b, 1.5).unwrap();
        audio.play(b, false).unwrap();
        audio.set_volume(a, 0.25).unwrap();

        assert_eq!(
            *log.lock().unwrap(),
            vec![
                ("gain", a, 0.5),
                ("play", a, 0.0),
                ("gain", a, 0.2),
                ("gain", b, 0.4),
                ("play", b, 0.0),
                ("gain", a, 0.1),
            ]
        );
        assert!(audio.set_volume(99, 0.5).is_err());
        audio.close(a).unwrap();
        assert_eq!(audio.volume(a), 1.0);
    }
}
