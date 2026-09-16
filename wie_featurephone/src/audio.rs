use core::cell::Cell;

use web_sys::{AudioContext, GainNode};

use wie_backend::{AudioCommand, AudioEventData, AudioSink};

/// WebAudio-backed sink. PCM waveforms are scheduled through the shared
/// `AudioContext`; MIDI events are accepted but not yet synthesized (silent
/// stub) — a soft-synth can be layered on later without touching the core.
///
/// Consecutive PCM chunks are scheduled back-to-back on a moving cursor
/// (`next_time`) instead of all at "now", so streamed game audio plays gaplessly
/// rather than overlapping into noise.
///
/// The `AudioContext` must be created and resumed by the JS side on a user
/// gesture (browser autoplay policy). When no context is supplied, every method
/// is a no-op.
pub struct WebAudioSink {
    ctx: Option<AudioContext>,
    // Output node: the JS-owned master gain (gain → destination). When present,
    // PCM is routed through it so the UI volume control governs the real output.
    gain: Option<GainNode>,
    // Next free playback position on the audio timeline (seconds).
    next_time: Cell<f64>,
}

// Single-threaded in the browser; the JS handle never crosses threads.
unsafe impl Send for WebAudioSink {}
unsafe impl Sync for WebAudioSink {}

impl WebAudioSink {
    pub fn new(ctx: Option<AudioContext>, gain: Option<GainNode>) -> Self {
        Self {
            ctx,
            gain,
            next_time: Cell::new(0.0),
        }
    }

    fn try_play(&self, ctx: &AudioContext, start_at: f64, channels: u8, sampling_rate: u32, wave_data: &[i16]) -> Result<(), wasm_bindgen::JsValue> {
        let channels = channels.max(1) as u32;
        if sampling_rate == 0 || wave_data.is_empty() {
            return Ok(());
        }
        let frames = wave_data.len() as u32 / channels;
        if frames == 0 {
            return Ok(());
        }

        let buffer = ctx.create_buffer(channels, frames, sampling_rate as f32)?;
        for ch in 0..channels {
            let samples: Vec<f32> = (0..frames)
                .map(|frame| {
                    let idx = (frame * channels + ch) as usize;
                    wave_data[idx] as f32 / 32768.0
                })
                .collect();
            buffer.copy_to_channel(&samples, ch as i32)?;
        }

        let source = ctx.create_buffer_source()?;
        source.set_buffer(Some(&buffer));
        // Route through the master gain when available, else straight to output.
        match &self.gain {
            Some(gain) => {
                source.connect_with_audio_node(gain)?;
            }
            None => {
                source.connect_with_audio_node(&ctx.destination())?;
            }
        }

        source.start_with_when(start_at)?;
        let duration = frames as f64 / sampling_rate as f64;
        // Keep the cursor at the furthest scheduled end so a following sequence
        // queues after this one instead of overlapping it.
        self.next_time.set(self.next_time.get().max(start_at + duration));
        Ok(())
    }
}

impl AudioSink for WebAudioSink {
    fn send(&self, command: AudioCommand) {
        let Some(ctx) = self.ctx.as_ref() else { return };

        match command {
            AudioCommand::Play { sequence, .. } => {
                // Schedule at the later of "now" and the running cursor. If the
                // cursor fell behind (underrun), resync to now to avoid a
                // growing delay. Event times are ms offsets within the sequence.
                let base = self.next_time.get().max(ctx.current_time());
                for event in &sequence.events {
                    if let AudioEventData::Wave {
                        channels,
                        sampling_rate,
                        samples,
                    } = &event.data
                    {
                        // Errors are non-fatal: a failed audio schedule must
                        // never abort the emulation tick.
                        let _ = self.try_play(ctx, base + event.time as f64 / 1000.0, *channels, *sampling_rate, samples);
                    }
                }
            }
            // ponytail: MIDI events and Stop/repeat are dropped, matching what
            // this host did before the sequence API existed (MIDI was always a
            // silent stub; there was no stop or loop at all). Upgrade path is
            // upstream's own browser host: a JS-side AudioPlayer that owns
            // per-handle scheduling (`wie-web/src/rust/audio_sink.rs` +
            // `midi.ts`). That is a JS-surface addition, not an adapter change.
            AudioCommand::Stop { .. } => {}
        }
    }
}
