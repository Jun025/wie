use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
};

use js_sys::{Array, ArrayBuffer, Int16Array, Object, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{AudioContext, AudioWorkletNode, AudioWorkletNodeOptions, Blob, BlobPropertyBag, GainNode, MessageEvent, Response, Url};

use wie_backend::{AudioCommand, AudioEventData, AudioHandle, AudioSink};

/// The synth and scheduler, run on the audio thread. Shipped inside the wasm as a string and
/// loaded from a Blob URL, because the engine artifact is exactly two files
/// (`docs/contracts/featurephone-engine-contract.json` `artifacts.files`) — a third file would be
/// a coordinated wie + otterpebble contract change.
const WORKLET_SOURCE: &str = include_str!("audio_worklet.js");
const PROCESSOR_NAME: &str = "wie-audio";
/// spessasynth_core, bundled by `scripts/build-soundfont-prelude.mjs` and embedded by `build.rs`;
/// empty in any build that did not go through `scripts/build-wasm.sh`. It only publishes
/// `globalThis.wieSoundfont`, and is loaded as a SECOND module into the worklet's global scope once
/// the soundfont file has arrived — never with [`WORKLET_SOURCE`]. Evaluating it costs ~100 ms of
/// audio-thread CPU (stb-vorbis decodes its embedded wasm from base64 without `atob`, which the
/// `AudioWorkletGlobalScope` lacks); until 2026-09-28 it rode in the same Blob, so with a URL the
/// first `Play` queued behind that evaluation. Loaded afterwards, the first sound cannot wait for it.
const SOUNDFONT_PRELUDE: &str = include_str!(concat!(env!("OUT_DIR"), "/soundfont_prelude.js"));

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
///
/// With a soundfont URL (and a build carrying [`SOUNDFONT_PRELUDE`]), the sink starts a background
/// fetch of that file when it is created — the engine's boot — and tells the worklet one is coming
/// (`sfwait`) before any play. When it arrives, the prelude is loaded as a second worklet module and
/// the file is handed to the worklet. Every MIDI play of the session renders through the soundfont,
/// the first included: the worklet holds a play until it can (bounded — see `HOLD_MAX_MS` there).
/// Until 2026-10-04 the fetch started after the first `Play`, so each song's first play was FM and
/// its next the soundfont (docs/report 0438). Any failure — fetch, HTTP status, prelude load, parse —
/// is posted as `sfoff` and makes the whole session FM, which is what no URL means too. The worklet
/// module is byte-for-byte the same URL or not; only the prelude waits for the file.
pub struct WebAudioSink {
    state: Option<Rc<RefCell<State>>>,
}

struct State {
    ctx: AudioContext,
    gain: Option<GainNode>,
    mode: Mode,
    /// Commands received while the worklet module is still loading.
    queue: Vec<Queued>,
    /// Handles whose events the worklet holds, least recently played first — a handle's sequence
    /// never changes, so a replay sends only the handle. Capped at [`RESIDENT_SEQUENCES`].
    loaded: VecDeque<u32>,
    /// Fallback only: next free playback position on the audio timeline (seconds).
    next_time: Cell<f64>,
    soundfont: Soundfont,
}

/// `Off` is terminal. Otherwise: `Requested` (at creation) → (file in hand) `Arrived` while the
/// worklet module still loads, else straight on → `Prelude` → `Posted`. A failure anywhere ends the
/// chain in `Off` with a warning and an `sfoff` to the worklet; `scripts/contract-roundtrip.mjs`
/// Scenario S drives every arrow of this in a browser.
enum Soundfont {
    /// No URL, a build without the prelude, or a failure: FM only.
    Off,
    /// Fetching; the worklet is told to wait for it (`sfwait`) as soon as it exists.
    Requested,
    /// Arrived while the worklet module was still loading; the prelude load starts once the node exists.
    Arrived(ArrayBuffer),
    /// The prelude module is loading; the file is posted when it resolves.
    Prelude,
    Posted,
}

enum Queued {
    Command(AudioCommand),
    Gain(AudioHandle, f32),
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
    pub fn new(ctx: Option<AudioContext>, gain: Option<GainNode>, soundfont_url: Option<String>) -> Self {
        let Some(ctx) = ctx else { return Self { state: None } };

        let fetch = match soundfont_url {
            Some(url) if !url.is_empty() && !SOUNDFONT_PRELUDE.is_empty() => Some(url),
            Some(url) if !url.is_empty() => {
                soundfont_log(false, "URL given, but this build has no soundfont prelude");
                None
            }
            _ => None,
        };
        let soundfont = if fetch.is_some() { Soundfont::Requested } else { Soundfont::Off };
        let state = Rc::new(RefCell::new(State {
            ctx,
            gain,
            mode: Mode::Loading,
            queue: Vec::new(),
            loaded: VecDeque::new(),
            next_time: Cell::new(0.0),
            soundfont,
        }));

        if let Err(error) = start_worklet(&state) {
            tracing::warn!("audio worklet unavailable, MIDI will be silent: {error:?}");
            state.borrow_mut().fall_back();
        }
        // At boot, not at the first play: the worklet holds MIDI plays until the soundfont is in, so
        // the sooner it arrives the less the first song waits.
        if let Some(url) = fetch
            && let Err(error) = fetch_soundfont(&state, &url)
        {
            state.borrow_mut().give_up(&format!("fetch could not start: {error:?}"));
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
            Mode::Loading => state.queue.push(Queued::Command(command)),
            Mode::Worklet(_) => state.post(&command),
            Mode::Fallback => state.play_pcm(&command),
        }
    }

    // The fallback plays PCM at full gain: it is the no-worklet path, already missing MIDI.
    fn set_gain(&self, handle: AudioHandle, gain: f32) {
        let Some(state) = &self.state else { return };
        let mut state = state.borrow_mut();
        match &state.mode {
            Mode::Loading => state.queue.push(Queued::Gain(handle, gain)),
            Mode::Worklet(_) => state.post_gain(handle, gain),
            Mode::Fallback => {}
        }
    }
}

fn start_worklet(state: &Rc<RefCell<State>>) -> Result<(), JsValue> {
    let ctx = state.borrow().ctx.clone();
    let worklet = ctx.audio_worklet()?;

    // Alone, with or without a URL: the prelude is a later module (see `load_prelude`).
    let url = module_url(WORKLET_SOURCE)?;
    let promise = worklet.add_module(&url)?;

    let ready_state = state.clone();
    let ready_url = url.clone();
    let on_ready = Closure::once(move |_: JsValue| {
        let _ = Url::revoke_object_url(&ready_url);
        let mut state = ready_state.borrow_mut();
        match state.create_node() {
            Ok(node) => {
                state.mode = Mode::Worklet(node);
                // Before any play: from here the worklet holds MIDI plays for the soundfont.
                if !matches!(state.soundfont, Soundfont::Off) {
                    state.post_tag("sfwait");
                }
                for queued in core::mem::take(&mut state.queue) {
                    match queued {
                        Queued::Command(command) => state.post(&command),
                        Queued::Gain(handle, gain) => state.post_gain(handle, gain),
                    }
                }
                // Only a buffer that arrived while loading moves on — any other state stays put
                // (swapping unconditionally here once turned a not-yet-fetched soundfont into
                // `Posted`, so it was never fetched; caught by the browser run — and now by
                // contract-roundtrip.mjs Scenario S3).
                if matches!(state.soundfont, Soundfont::Arrived(_))
                    && let Soundfont::Arrived(buffer) = core::mem::replace(&mut state.soundfont, Soundfont::Prelude)
                {
                    load_prelude(&ready_state, &mut state, buffer);
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

fn module_url(source: &str) -> Result<String, JsValue> {
    let options = BlobPropertyBag::new();
    options.set_type("text/javascript");
    let blob = Blob::new_with_str_sequence_and_options(&Array::of1(&JsValue::from_str(source)), &options)?;
    Url::create_object_url_with_blob(&blob)
}

/// Loads [`SOUNDFONT_PRELUDE`] as a second module into the worklet's global scope (where it
/// publishes `globalThis.wieSoundfont` for the processor that already exists), then hands the
/// soundfont to the worklet. Called with the node created and `state.soundfont` already `Prelude`.
fn load_prelude(shared: &Rc<RefCell<State>>, state: &mut State, buffer: ArrayBuffer) {
    let promise = state
        .ctx
        .audio_worklet()
        .and_then(|worklet| module_url(SOUNDFONT_PRELUDE).and_then(|url| Ok((worklet.add_module(&url)?, url))));
    let (promise, url) = match promise {
        Ok(started) => started,
        Err(error) => return state.give_up(&format!("prelude could not load: {error:?}")),
    };
    let loaded_state = shared.clone();
    let loaded_url = url.clone();
    let on_loaded = Closure::once(move |_: JsValue| {
        let _ = Url::revoke_object_url(&loaded_url);
        let mut state = loaded_state.borrow_mut();
        state.soundfont = Soundfont::Posted;
        state.post_soundfont(&buffer);
    });
    let failed_state = shared.clone();
    let on_error = Closure::once(move |error: JsValue| {
        let _ = Url::revoke_object_url(&url);
        failed_state.borrow_mut().give_up(&format!("prelude failed to load: {error:?}"));
    });
    let _ = promise.then2(&on_loaded, &on_error);
    // One-shot callbacks owned by the promise from here on.
    on_loaded.forget();
    on_error.forget();
}

/// The soundfont's outcome, on the page console: this crate has no tracing subscriber, and the
/// shell needs to see why a soundfont it served is not heard. Never `console.error` — a missing
/// soundfont is a degraded sound, not a failure (verify-browser.mjs fails a deploy on errors).
fn soundfont_log(ok: bool, message: &str) {
    let line = JsValue::from_str(&format!("[wie] soundfont {message}{}", if ok { "" } else { " — playing FM" }));
    if ok {
        web_sys::console::log_1(&line)
    } else {
        web_sys::console::warn_1(&line)
    }
}

/// GETs the soundfont and hands it to the worklet. Every failure ends in [`State::give_up`] — the
/// FM path, for the whole session.
fn fetch_soundfont(state: &Rc<RefCell<State>>, url: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let arrived_state = state.clone();
    let response_state = state.clone();
    let fetch_state = state.clone();
    let on_response = Closure::once(move |response: JsValue| {
        let body = match response.dyn_into::<Response>() {
            Ok(response) if response.ok() => response.array_buffer(),
            Ok(response) => Err(JsValue::from_str(&format!("HTTP {}", response.status()))),
            Err(error) => Err(error),
        };
        let body = match body {
            Ok(body) => body,
            Err(error) => return response_state.borrow_mut().give_up(&format!("fetch failed: {error:?}")),
        };
        let on_buffer = Closure::once(move |buffer: JsValue| {
            let Ok(buffer) = buffer.dyn_into::<ArrayBuffer>() else { return };
            let mut state = arrived_state.borrow_mut();
            match &state.mode {
                Mode::Loading => state.soundfont = Soundfont::Arrived(buffer),
                Mode::Worklet(_) => {
                    state.soundfont = Soundfont::Prelude;
                    load_prelude(&arrived_state, &mut state, buffer);
                }
                Mode::Fallback => {}
            }
        });
        let on_error = Closure::once(move |error: JsValue| response_state.borrow_mut().give_up(&format!("download failed: {error:?}")));
        let _ = body.then2(&on_buffer, &on_error);
        on_buffer.forget();
        on_error.forget();
    });
    let on_error = Closure::once(move |error: JsValue| fetch_state.borrow_mut().give_up(&format!("fetch failed: {error:?}")));
    let _ = window.fetch_with_str(url).then2(&on_response, &on_error);
    // One-shot callbacks owned by the promise chain from here on.
    on_response.forget();
    on_error.forget();
    Ok(())
}

impl State {
    /// Transfers (not copies) the buffer to the audio thread. The worklet replies once with
    /// `{ t: "sf", ok, … }`; that reply is only logged — FM stays the answer to anything but `ok`.
    fn post_soundfont(&self, buffer: &ArrayBuffer) {
        let Mode::Worklet(node) = &self.mode else { return };
        let Ok(port) = node.port() else { return };
        let on_message = Closure::<dyn FnMut(MessageEvent)>::new(|event: MessageEvent| {
            let data = event.data();
            let field = |key: &str| Reflect::get(&data, &JsValue::from_str(key)).unwrap_or(JsValue::UNDEFINED);
            if field("t").as_string().as_deref() != Some("sf") {
                return;
            }
            if field("ok").as_bool() == Some(true) {
                soundfont_log(
                    true,
                    &format!(
                        "ready (parsed in {} ms) — plays that start from now use it",
                        field("ms").as_f64().unwrap_or(-1.0)
                    ),
                );
            } else {
                soundfont_log(false, &format!("not usable: {}", field("error").as_string().unwrap_or_default()));
            }
        });
        port.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        // Lives as long as the port; one per sink.
        on_message.forget();
        let message = Object::new();
        let _ = Reflect::set(&message, &JsValue::from_str("t"), &JsValue::from_str("sf"));
        let _ = Reflect::set(&message, &JsValue::from_str("data"), buffer);
        let _ = port.post_message_with_transferable(&message, &Array::of1(buffer));
    }

    /// The soundfont will not come: logged, and the worklet (if it exists yet) stops holding plays
    /// for it. Before the node exists, `Off` keeps `sfwait` from ever being sent.
    fn give_up(&mut self, message: &str) {
        soundfont_log(false, message);
        let waiting = !matches!(self.soundfont, Soundfont::Off);
        self.soundfont = Soundfont::Off;
        if waiting {
            self.post_tag("sfoff");
        }
    }

    /// A message that is just its tag (`sfwait`, `sfoff`).
    fn post_tag(&self, tag: &str) {
        let Mode::Worklet(node) = &self.mode else { return };
        let Ok(port) = node.port() else { return };
        let message = Object::new();
        let _ = Reflect::set(&message, &JsValue::from_str("t"), &JsValue::from_str(tag));
        let _ = port.post_message(&message);
    }

    fn fall_back(&mut self) {
        self.mode = Mode::Fallback;
        for queued in core::mem::take(&mut self.queue) {
            if let Queued::Command(command) = queued {
                self.play_pcm(&command);
            }
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

    fn post_gain(&self, handle: AudioHandle, gain: f32) {
        let Mode::Worklet(node) = &self.mode else { return };
        let Ok(port) = node.port() else { return };
        let message = Object::new();
        let _ = Reflect::set(&message, &JsValue::from_str("t"), &JsValue::from_str("gain"));
        let _ = Reflect::set(&message, &JsValue::from_str("h"), &JsValue::from(handle));
        let _ = Reflect::set(&message, &JsValue::from_str("g"), &JsValue::from_f64(gain as f64));
        let _ = port.post_message(&message);
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
