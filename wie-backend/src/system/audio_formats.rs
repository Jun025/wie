//! The audio a general MIDP game hands `Manager.createPlayer`, as an `AudioSequence`: Standard MIDI
//! Files (`audio/midi`), PCM WAV (`audio/x-wav`), and the tone device (`Manager.playTone`,
//! `ToneControl.setSequence`). Carrier titles only ever passed SMAF, so until 2026-10-08 every
//! other type was a `MediaException` and open-source MIDP games played silent.

use alloc::{vec, vec::Vec};

use crate::{AudioEventData, AudioSequence, TimedAudioEvent};

fn sequence(mut events: Vec<TimedAudioEvent>, end: u64) -> AudioSequence {
    events.sort_by_key(|x| x.time); // stable: same-time events keep file order
    let duration = events.iter().map(|x| x.time).max().unwrap_or(0).max(end);

    AudioSequence { duration, events }
}

fn read_var(data: &[u8], pos: &mut usize) -> Option<u32> {
    let mut value = 0u32;
    for _ in 0..4 {
        let byte = *data.get(*pos)?;
        *pos += 1;
        value = (value << 7) | (byte & 0x7f) as u32;
        if byte & 0x80 == 0 {
            return Some(value);
        }
    }

    None
}

/// A Standard MIDI File (format 0 or 1). Tempo changes apply across tracks, as in the format.
/// `None` when it is not one.
/// ponytail: SMPTE time division is read as 480 ticks a quarter note — no handset title seen uses it.
pub fn parse_midi(data: &[u8]) -> Option<AudioSequence> {
    if data.get(..4)? != b"MThd" {
        return None;
    }
    let header_len = u32::from_be_bytes(data.get(4..8)?.try_into().ok()?) as usize;
    let division = u16::from_be_bytes(data.get(12..14)?.try_into().ok()?);
    let ticks_per_quarter = if division & 0x8000 != 0 || division == 0 { 480 } else { division as u64 };

    // (tick, order, message) for channel messages and (tick, tempo) for tempo changes
    let mut messages: Vec<(u64, usize, Vec<u8>)> = Vec::new();
    let mut tempos: Vec<(u64, u64)> = Vec::new();
    let mut end_tick = 0u64;

    let mut pos = 8 + header_len;
    while let Some(chunk) = data.get(pos..pos + 8) {
        let len = u32::from_be_bytes(chunk[4..8].try_into().ok()?) as usize;
        let body = data.get(pos + 8..(pos + 8 + len).min(data.len()))?;
        pos += 8 + len;
        if &chunk[..4] != b"MTrk" {
            continue;
        }

        let (mut at, mut tick, mut status) = (0usize, 0u64, 0u8);
        while at < body.len() {
            let Some(delta) = read_var(body, &mut at) else { break };
            tick += delta as u64;
            let Some(&first) = body.get(at) else { break };
            if first & 0x80 != 0 {
                at += 1;
                status = first;
            }
            match status {
                0xff => {
                    let Some(&kind) = body.get(at) else { break };
                    at += 1;
                    let Some(len) = read_var(body, &mut at) else { break };
                    let meta = body.get(at..at + len as usize).unwrap_or(&[]);
                    if kind == 0x51 && meta.len() == 3 {
                        tempos.push((tick, u32::from_be_bytes([0, meta[0], meta[1], meta[2]]) as u64));
                    }
                    at += len as usize;
                    status = 0; // meta and sysex cancel running status
                }
                0xf0 | 0xf7 => {
                    let Some(len) = read_var(body, &mut at) else { break };
                    at += len as usize;
                    status = 0;
                }
                0x80..=0xef => {
                    let len = if matches!(status & 0xf0, 0xc0 | 0xd0) { 1 } else { 2 };
                    let Some(bytes) = body.get(at..at + len) else { break };
                    at += len;
                    let mut message = vec![status];
                    message.extend_from_slice(bytes);
                    messages.push((tick, messages.len(), message));
                }
                _ => break, // data byte with no running status: the track is damaged, keep what we have
            }
        }
        end_tick = end_tick.max(tick);
    }

    tempos.sort_by_key(|x| x.0);
    let to_ms = |tick: u64| {
        // walk the tempo map: microseconds per quarter note, 500,000 (120 bpm) until the first change
        let (mut ms_x_tpq, mut last_tick, mut tempo) = (0u128, 0u64, 500_000u64);
        for &(at, next) in tempos.iter().take_while(|x| x.0 <= tick) {
            ms_x_tpq += (at - last_tick) as u128 * tempo as u128;
            last_tick = at;
            tempo = next;
        }
        ms_x_tpq += (tick - last_tick) as u128 * tempo as u128;

        (ms_x_tpq / (ticks_per_quarter as u128 * 1000)) as u64
    };

    messages.sort_by_key(|x| (x.0, x.1));
    let events = messages
        .into_iter()
        .map(|(tick, _, message)| TimedAudioEvent {
            time: to_ms(tick),
            data: AudioEventData::Midi(message),
        })
        .collect();

    Some(sequence(events, to_ms(end_tick)))
}

/// A RIFF/WAVE file of 8- or 16-bit PCM. `None` when it is anything else (ADPCM, float, not WAV).
pub fn parse_wav(data: &[u8]) -> Option<AudioSequence> {
    if data.get(..4)? != b"RIFF" || data.get(8..12)? != b"WAVE" {
        return None;
    }

    let (mut format, mut samples) = (None, None);
    let mut pos = 12;
    while let Some(chunk) = data.get(pos..pos + 8) {
        let len = u32::from_le_bytes(chunk[4..8].try_into().ok()?) as usize;
        let body = &data[pos + 8..(pos + 8 + len).min(data.len())];
        match &chunk[..4] {
            b"fmt " if body.len() >= 16 => {
                let tag = u16::from_le_bytes([body[0], body[1]]);
                let channels = u16::from_le_bytes([body[2], body[3]]);
                let rate = u32::from_le_bytes(body[4..8].try_into().ok()?);
                let bits = u16::from_le_bytes([body[14], body[15]]);
                format = Some((tag, channels, rate, bits));
            }
            b"data" => samples = Some(body),
            _ => {}
        }
        pos += 8 + len + (len & 1); // chunks are word-aligned
    }

    let ((1, channels @ 1..=2, rate @ 1.., bits), Some(body)) = (format?, samples) else {
        return None;
    };
    let samples: Vec<i16> = match bits {
        8 => body.iter().map(|&x| ((x as i16) - 128) << 8).collect(),
        16 => body.as_chunks::<2>().0.iter().map(|&x| i16::from_le_bytes(x)).collect(),
        _ => return None,
    };
    let length_ms = samples.len() as u64 * 1000 / (rate as u64 * channels as u64);

    Some(sequence(
        vec![TimedAudioEvent {
            time: 0,
            data: AudioEventData::Wave {
                channels: channels as u8,
                sampling_rate: rate,
                samples,
            },
        }],
        length_ms,
    ))
}

// The tone device's voice: GM program 80, «Lead 1 (square)» — the beeper a handset's tone generator is.
const TONE_PROGRAM: u8 = 80;

fn tone_events(events: &mut Vec<TimedAudioEvent>, at: u64, note: u8, length: u64, volume: u8) {
    let velocity = ((volume.min(100) as u32 * 127) / 100) as u8;
    events.push(TimedAudioEvent {
        time: at,
        data: AudioEventData::Midi(vec![0x90, note, velocity]),
    });
    events.push(TimedAudioEvent {
        time: at + length,
        data: AudioEventData::Midi(vec![0x80, note, 0]),
    });
}

fn tone_program() -> TimedAudioEvent {
    TimedAudioEvent {
        time: 0,
        data: AudioEventData::Midi(vec![0xc0, TONE_PROGRAM]),
    }
}

/// `Manager.playTone(note, duration, volume)`: one note, `volume` 0..100.
pub fn tone(note: i32, duration_ms: i32, volume: i32) -> AudioSequence {
    let mut events = vec![tone_program()];
    let length = duration_ms.max(0) as u64;
    if volume > 0 {
        tone_events(&mut events, 0, note.clamp(0, 127) as u8, length, volume.clamp(0, 100) as u8);
    }

    sequence(events, length)
}

/// A JSR-135 tone sequence (`ToneControl.setSequence`): VERSION, TEMPO, RESOLUTION, BLOCK_START/END,
/// PLAY_BLOCK, SET_VOLUME, REPEAT, SILENCE and notes. `None` when it does not parse.
pub fn parse_tone_sequence(data: &[i8]) -> Option<AudioSequence> {
    const VERSION: i8 = -2;
    const TEMPO: i8 = -3;
    const RESOLUTION: i8 = -4;
    const BLOCK_START: i8 = -5;
    const BLOCK_END: i8 = -6;
    const PLAY_BLOCK: i8 = -7;
    const SET_VOLUME: i8 = -8;
    const REPEAT: i8 = -9;
    const SILENCE: i8 = -1;

    let mut pos = 0;
    let mut pairs = || -> Option<(i8, i8)> {
        let pair = (*data.get(pos)?, *data.get(pos + 1)?);
        pos += 2;
        Some(pair)
    };

    let (mut tempo, mut resolution) = (120i64, 64i64);
    let mut blocks: [Option<Vec<(i8, i8)>>; 128] = [const { None }; 128];
    let mut body: Vec<(i8, i8)> = Vec::new();
    let mut open_block: Option<(usize, Vec<(i8, i8)>)> = None;
    while let Some((a, b)) = pairs() {
        match a {
            VERSION => {}
            TEMPO => tempo = (b as i64).clamp(5, 127) * 4,
            RESOLUTION => resolution = (b as i64).max(1),
            BLOCK_START => open_block = Some((b.max(0) as usize, Vec::new())),
            BLOCK_END => {
                let (number, block) = open_block.take()?;
                blocks[number] = Some(block);
            }
            _ => match &mut open_block {
                Some((_, block)) => block.push((a, b)),
                None => body.push((a, b)),
            },
        }
    }

    // a «note» lasts `duration` units of 1/resolution of a whole note, at `tempo` quarter notes a minute
    let unit_ms = |duration: i8| (duration.max(0) as i64 * 240_000 / (resolution * tempo)).max(0) as u64;
    let mut events = vec![tone_program()];
    let (mut at, mut volume, mut repeat) = (0u64, 100u8, 1usize);
    let mut stack = vec![body.into_iter()];
    while let Some(top) = stack.last_mut() {
        let Some((a, b)) = top.next() else {
            stack.pop();
            continue;
        };
        match a {
            SET_VOLUME => volume = b.clamp(0, 100) as u8,
            REPEAT => repeat = b.max(1) as usize,
            PLAY_BLOCK => {
                let block = blocks.get(b.max(0) as usize)?.clone()?;
                if stack.len() < 8 {
                    stack.push(block.into_iter());
                }
            }
            SILENCE => at += unit_ms(b) * repeat as u64,
            0.. => {
                for _ in 0..core::mem::replace(&mut repeat, 1) {
                    let length = unit_ms(b);
                    if volume > 0 {
                        tone_events(&mut events, at, a as u8, length, volume);
                    }
                    at += length;
                }
            }
            _ => return None,
        }
    }

    Some(sequence(events, at))
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::{parse_midi, parse_tone_sequence, parse_wav, tone};
    use crate::{AudioEventData, TimedAudioEvent};

    fn midi(time: u64, bytes: &[u8]) -> TimedAudioEvent {
        TimedAudioEvent {
            time,
            data: AudioEventData::Midi(bytes.to_vec()),
        }
    }

    #[test]
    fn midi_tempo_running_status_and_two_tracks() {
        // 96 ticks a quarter; track 1 sets 1,000,000 µs/quarter (60 bpm) at tick 96
        let mut smf = b"MThd\0\0\0\x06\0\x01\0\x02\0\x60".to_vec();
        let t1 = [0x00, 0xc0, 0x05, 0x60, 0xff, 0x51, 0x03, 0x0f, 0x42, 0x40, 0x00, 0xff, 0x2f, 0x00];
        smf.extend(b"MTrk");
        smf.extend((t1.len() as u32).to_be_bytes());
        smf.extend(t1);
        // note on at 0, running-status note on at 96, note off at 192
        let t2 = [0x00, 0x90, 0x3c, 0x40, 0x60, 0x3e, 0x40, 0x60, 0x80, 0x3c, 0x00, 0x00, 0xff, 0x2f, 0x00];
        smf.extend(b"MTrk");
        smf.extend((t2.len() as u32).to_be_bytes());
        smf.extend(t2);

        let sequence = parse_midi(&smf).unwrap();
        assert_eq!(
            sequence.events,
            vec![
                midi(0, &[0xc0, 0x05]),
                midi(0, &[0x90, 0x3c, 0x40]),
                midi(500, &[0x90, 0x3e, 0x40]),  // first quarter at 120 bpm
                midi(1500, &[0x80, 0x3c, 0x00]), // second at 60 bpm
            ]
        );
        assert_eq!(sequence.duration, 1500);
        assert!(parse_midi(b"MMMD").is_none());
    }

    #[test]
    fn wav_8_and_16_bit() {
        let wav = |bits: u16, data: &[u8]| {
            let mut w = b"RIFF\0\0\0\0WAVEfmt \x10\0\0\0\x01\0\x01\0".to_vec();
            w.extend(8000u32.to_le_bytes());
            w.extend((8000u32 * bits as u32 / 8).to_le_bytes());
            w.extend((bits / 8).to_le_bytes());
            w.extend(bits.to_le_bytes());
            w.extend(b"data");
            w.extend((data.len() as u32).to_le_bytes());
            w.extend(data);
            w
        };

        let eight = parse_wav(&wav(8, &[128, 255, 0])).unwrap();
        let AudioEventData::Wave { samples, sampling_rate, .. } = &eight.events[0].data else {
            panic!()
        };
        assert_eq!((samples.as_slice(), *sampling_rate), ([0, 127 << 8, -128 << 8].as_slice(), 8000));

        let sixteen = parse_wav(&wav(16, &[0x34, 0x12, 0xff, 0xff])).unwrap();
        let AudioEventData::Wave { samples, .. } = &sixteen.events[0].data else {
            panic!()
        };
        assert_eq!(samples.as_slice(), [0x1234, -1]);
        assert!(parse_wav(b"RIFF\0\0\0\0WAVE").is_none());
    }

    #[test]
    fn tones() {
        let one = tone(60, 250, 100);
        assert_eq!(
            one.events,
            vec![midi(0, &[0xc0, 80]), midi(0, &[0x90, 60, 127]), midi(250, &[0x80, 60, 0])]
        );
        assert_eq!(one.duration, 250);

        // VERSION 1, TEMPO 30 (=120 bpm), block 0 = C4 quarter; play it twice around a silence, then a repeated D4 eighth
        let seq: [i8; 20] = [-2, 1, -3, 30, -5, 0, 60, 16, -6, 0, -7, 0, -1, 8, -7, 0, -9, 2, 62, 8];
        let parsed = parse_tone_sequence(&seq).unwrap();
        let ons: vec::Vec<u64> = parsed
            .events
            .iter()
            .filter(|x| matches!(&x.data, AudioEventData::Midi(m) if m[0] == 0x90))
            .map(|x| x.time)
            .collect();
        assert_eq!(ons, [0, 750, 1250, 1500]);
        assert_eq!(parsed.duration, 1750);
        assert!(parse_tone_sequence(&[-5, 0, 60, 16]).is_some()); // an unclosed block is just not played
        assert!(parse_tone_sequence(&[-7, 3]).is_none()); // undefined block
    }
}
