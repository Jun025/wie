use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
};

use js_sys::{Array, Int16Array, Object, Reflect, Uint8Array};
use wasm_bindgen::{JsValue, closure::Closure};
use web_sys::{AudioContext, AudioWorkletNode, AudioWorkletNodeOptions, Blob, BlobPropertyBag, GainNode, Url};

use wie_backend::{AudioCommand, AudioEventData, AudioSink};

/// The synth and scheduler, run on the audio thread. Shipped inside the wasm as a string and
/// loaded from a Blob URL, because the engine artifact is exactly two files
/// (`docs/contracts/featurephone-engine-contract.json` `artifacts.files`) — a third file would be
/// a coordinated wie + otterpebble contract change.
const WORKLET_SOURCE: &str = include_str!("audio_worklet.js");
const PROCESSOR_NAME: &str = "wie-audio";

/// How many handles' sequences the worklet keeps. A handle's events cross to the audio thread once
/// and are reused by every later play, but `AudioCommand` has no "close", so the sink never learns
/// that a handle is retired — and handles are never reused (`Audio` counts up). Until 2026-09-27
/// every sequence ever played stayed in the worklet: 더팜1 opens a fresh handle per sound effect
/// (27 in 60 s, PCM included), so a long session grew without bound. Past this many, the least
/// recently played sequence is dropped; playing it again just resends its events.
///
/// Why not a close command: `AudioCommand` is upstream's, and its other sink (`wie-web`) matches
/// it exhaustively, so a new variant breaks that crate. Eviction needs nothing from anyone else,
/// and it is exact: the worklet's playback holds its own reference to the sequence, so dropping it
/// never cuts a sound that is playing. 32 is well above the live handles an SKT title holds (one per
/// clip object; the most in the 50-title corpus is 5), so in practice only retired handles go.
const RESIDENT_SEQUENCES: usize = 32;

/// WebAudio-backed sink.
///
/// Every command goes to an `AudioWorkletNode` (`audio_worklet.js`), which synthesizes MIDI,
/// plays PCM, and owns per-handle scheduling: `repeat` loops, `Stop` cuts that handle's notes and
/// samples, and handles mix (BGM under SFX). Until 2026-09-27 this sink played only `Wave`
/// events and dropped MIDI, `repeat` and `Stop` — measured on 배틀몬스터, 258 of its 272 audio
/// events were MIDI, so the background music and most effects were silent.
///
/// The worklet module loads asynchronously; commands that arrive before it is ready are queued
/// and replayed in order. If the browser cannot load it (no `AudioWorklet`, or `addModule`
/// rejects), the sink falls back to the pre-worklet behaviour: PCM through `AudioBufferSource`s,
/// MIDI silent — never worse than before.
///
/// The `AudioContext` must be created and resumed by the JS side on a user gesture (browser
/// autoplay policy); output goes through the JS-owned master gain so the UI volume governs MIDI
/// and PCM alike. When no context is supplied, every method is a no-op.
pub struct WebAudioSink {
    state: Option<Rc<RefCell<State>>>,
}

struct State {
    ctx: AudioContext,
    gain: Option<GainNode>,
    mode: Mode,
    /// Commands received while the worklet module is still loading.
    queue: Vec<AudioCommand>,
    /// Handles whose events the worklet holds, least recently played first — a handle's sequence
    /// never changes, so a replay sends only the handle. Capped at [`RESIDENT_SEQUENCES`].
    loaded: VecDeque<u32>,
    /// Fallback only: next free playback position on the audio timeline (seconds).
    next_time: Cell<f64>,
}

enum Mode {
    Loading,
    Worklet(AudioWorkletNode),
    Fallback,
}

// Single-threaded in the browser; the JS handles never cross threads.
unsafe impl Send for WebAudioSink {}
unsafe impl Sync for WebAudioSink {}

impl WebAudioSink {
    pub fn new(ctx: Option<AudioContext>, gain: Option<GainNode>) -> Self {
        let Some(ctx) = ctx else { return Self { state: None } };

        let state = Rc::new(RefCell::new(State {
            ctx,
            gain,
            mode: Mode::Loading,
            queue: Vec::new(),
            loaded: VecDeque::new(),
            next_time: Cell::new(0.0),
        }));

        if let Err(error) = start_worklet(&state) {
            tracing::warn!("audio worklet unavailable, MIDI will be silent: {error:?}");
            state.borrow_mut().fall_back();
        }

        Self { state: Some(state) }
    }
}

impl Drop for WebAudioSink {
    fn drop(&mut self) {
        if let Some(state) = &self.state
            && let Mode::Worklet(node) = &state.borrow().mode
        {
            let _ = node.disconnect();
        }
    }
}

impl AudioSink for WebAudioSink {
    fn send(&self, command: AudioCommand) {
        let Some(state) = &self.state else { return };
        let mut state = state.borrow_mut();
        match &state.mode {
            Mode::Loading => state.queue.push(command),
            Mode::Worklet(_) => state.post(&command),
            Mode::Fallback => state.play_pcm(&command),
        }
    }
}

fn start_worklet(state: &Rc<RefCell<State>>) -> Result<(), JsValue> {
    let ctx = state.borrow().ctx.clone();
    let worklet = ctx.audio_worklet()?;

    let options = BlobPropertyBag::new();
    options.set_type("text/javascript");
    let blob = Blob::new_with_str_sequence_and_options(&Array::of1(&JsValue::from_str(WORKLET_SOURCE)), &options)?;
    let url = Url::create_object_url_with_blob(&blob)?;
    let promise = worklet.add_module(&url)?;

    let ready_state = state.clone();
    let ready_url = url.clone();
    let on_ready = Closure::once(move |_: JsValue| {
        let _ = Url::revoke_object_url(&ready_url);
        let mut state = ready_state.borrow_mut();
        match state.create_node() {
            Ok(node) => {
                state.mode = Mode::Worklet(node);
                for command in core::mem::take(&mut state.queue) {
                    state.post(&command);
                }
            }
            Err(error) => {
                tracing::warn!("audio worklet node creation failed, MIDI will be silent: {error:?}");
                state.fall_back();
            }
        }
    });
    let failed_state = state.clone();
    let on_error = Closure::once(move |error: JsValue| {
        let _ = Url::revoke_object_url(&url);
        tracing::warn!("audio worklet module failed to load, MIDI will be silent: {error:?}");
        failed_state.borrow_mut().fall_back();
    });
    let _ = promise.then2(&on_ready, &on_error);
    // One-shot callbacks owned by the promise from here on.
    on_ready.forget();
    on_error.forget();

    Ok(())
}

impl State {
    fn fall_back(&mut self) {
        self.mode = Mode::Fallback;
        for command in core::mem::take(&mut self.queue) {
            self.play_pcm(&command);
        }
    }

    fn create_node(&self) -> Result<AudioWorkletNode, JsValue> {
        let options = AudioWorkletNodeOptions::new();
        options.set_number_of_inputs(0);
        options.set_number_of_outputs(1);
        options.set_output_channel_count(&Array::of1(&JsValue::from(2)));
        let node = AudioWorkletNode::new_with_options(&self.ctx, PROCESSOR_NAME, &options)?;
        match &self.gain {
            Some(gain) => node.connect_with_audio_node(gain)?,
            None => node.connect_with_audio_node(&self.ctx.destination())?,
        };
        Ok(node)
    }

    fn post(&mut self, command: &AudioCommand) {
        let Mode::Worklet(node) = &self.mode else { return };
        let Ok(port) = node.port() else { return };
        let mut evicted = None;
        let message = Object::new();
        let set = |key: &str, value: &JsValue| {
            let _ = Reflect::set(&message, &JsValue::from_str(key), value);
        };
        match command {
            AudioCommand::Play { handle, sequence, repeat } => {
                set("t", &JsValue::from_str("play"));
                set("h", &JsValue::from(*handle));
                set("r", &JsValue::from(*repeat));
                set("d", &JsValue::from_f64(sequence.duration as f64));
                let resident = match self.loaded.iter().position(|loaded| loaded == handle) {
                    Some(index) => {
                        self.loaded.remove(index);
                        true
                    }
                    None => false,
                };
                self.loaded.push_back(*handle);
                if self.loaded.len() > RESIDENT_SEQUENCES {
                    evicted = self.loaded.pop_front();
                }
                if !resident {
                    let events = Array::new();
                    for event in &sequence.events {
                        let entry = Array::new();
                        entry.push(&JsValue::from_f64(event.time as f64));
                        match &event.data {
                            AudioEventData::Midi(data) => {
                                entry.push(&JsValue::from(0));
                                entry.push(Uint8Array::from(data.as_slice()).as_ref());
                            }
                            AudioEventData::Wave {
                                channels,
                                sampling_rate,
                                samples,
                            } => {
                                entry.push(&JsValue::from(1));
                                entry.push(&JsValue::from(*channels));
                                entry.push(&JsValue::from(*sampling_rate));
                                entry.push(Int16Array::from(samples.as_slice()).as_ref());
                            }
                        }
                        events.push(&entry);
                    }
                    set("ev", &events);
                }
            }
            AudioCommand::Stop { handle } => {
                set("t", &JsValue::from_str("stop"));
                set("h", &JsValue::from(*handle));
            }
        }
        // A failed post must never abort the emulation tick.
        let _ = port.post_message(&message);
        if let Some(handle) = evicted {
            let evict = Object::new();
            let _ = Reflect::set(&evict, &JsValue::from_str("t"), &JsValue::from_str("evict"));
            let _ = Reflect::set(&evict, &JsValue::from_str("h"), &JsValue::from(handle));
            let _ = port.post_message(&evict);
        }
    }

    /// The pre-worklet path: PCM only, scheduled back-to-back on a moving cursor so streamed
    /// chunks play gaplessly. MIDI, `repeat` and `Stop` are dropped here.
    fn play_pcm(&self, command: &AudioCommand) {
        let AudioCommand::Play { sequence, .. } = command else { return };
        // Schedule at the later of "now" and the running cursor; if the cursor fell behind
        // (underrun), resync to now. Event times are ms offsets within the sequence.
        let base = self.next_time.get().max(self.ctx.current_time());
        for event in &sequence.events {
            if let AudioEventData::Wave {
                channels,
                sampling_rate,
                samples,
            } = &event.data
            {
                let _ = self.try_play(base + event.time as f64 / 1000.0, *channels, *sampling_rate, samples);
            }
        }
    }

    fn try_play(&self, start_at: f64, channels: u8, sampling_rate: u32, wave_data: &[i16]) -> Result<(), JsValue> {
        let channels = channels.max(1) as u32;
        if sampling_rate == 0 || wave_data.is_empty() {
            return Ok(());
        }
        let frames = wave_data.len() as u32 / channels;
        if frames == 0 {
            return Ok(());
        }

        let buffer = self.ctx.create_buffer(channels, frames, sampling_rate as f32)?;
        for ch in 0..channels {
            let samples: Vec<f32> = (0..frames)
                .map(|frame| wave_data[(frame * channels + ch) as usize] as f32 / 32768.0)
                .collect();
            buffer.copy_to_channel(&samples, ch as i32)?;
        }

        let source = self.ctx.create_buffer_source()?;
        source.set_buffer(Some(&buffer));
        match &self.gain {
            Some(gain) => source.connect_with_audio_node(gain)?,
            None => source.connect_with_audio_node(&self.ctx.destination())?,
        };

        source.start_with_when(start_at)?;
        let duration = frames as f64 / sampling_rate as f64;
        self.next_time.set(self.next_time.get().max(start_at + duration));
        Ok(())
    }
}
