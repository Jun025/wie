use web_sys::AudioContext;

use wie_backend::AudioSink;

/// WebAudio-backed sink. PCM waveforms are scheduled through the shared
/// `AudioContext`; MIDI events are accepted but not yet synthesized (silent
/// stub) — a soft-synth can be layered on later without touching the core.
///
/// The `AudioContext` must be created and resumed by the JS side on a user
/// gesture (browser autoplay policy). When no context is supplied, every method
/// is a no-op.
pub struct WebAudioSink {
    ctx: Option<AudioContext>,
}

// Single-threaded in the browser; the JS handle never crosses threads.
unsafe impl Send for WebAudioSink {}
unsafe impl Sync for WebAudioSink {}

impl WebAudioSink {
    pub fn new(ctx: Option<AudioContext>) -> Self {
        Self { ctx }
    }

    fn try_play(ctx: &AudioContext, channels: u8, sampling_rate: u32, wave_data: &[i16]) -> Result<(), wasm_bindgen::JsValue> {
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
        source.connect_with_audio_node(&ctx.destination())?;
        source.start()?;
        Ok(())
    }
}

impl AudioSink for WebAudioSink {
    fn play_wave(&self, channel: u8, sampling_rate: u32, wave_data: &[i16]) {
        if let Some(ctx) = self.ctx.as_ref() {
            // Errors are non-fatal: a failed audio schedule must never abort the
            // emulation tick.
            let _ = Self::try_play(ctx, channel, sampling_rate, wave_data);
        }
    }

    fn midi_note_on(&self, _channel_id: u8, _note: u8, _velocity: u8) {}
    fn midi_note_off(&self, _channel_id: u8, _note: u8, _velocity: u8) {}
    fn midi_program_change(&self, _channel_id: u8, _program: u8) {}
    fn midi_control_change(&self, _channel_id: u8, _control: u8, _value: u8) {}
}
