use alloc::{boxed::Box, vec, vec::Vec};
use core::sync::atomic::{AtomicI32, Ordering};

use bytemuck::{Pod, Zeroable};

use wipi_types::wipic::WIPICWord;

use wie_util::{Result, WieError, read_generic, write_generic};

use crate::{WIPICResult, context::WIPICContext, method::MethodBody};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MdaClip {
    clip_id: i32,
    h_proc: i32,
    r#type: u8,
    in_use: u8, // bool
    _padding1: [u8; 2],
    dev_id: i32,

    x: i32,
    y: i32,
    w: i32,
    h: i32,
    mute: u8, // bool
    _padding2: [u8; 3],
    watermark: i32,
    position: i32,
    quality: i32,
    mode: i32,
    state: i32,
    penpot: i32,
    num_slave: i32,

    clip_save: WIPICWord, // MC_MdaClip**

    audio_tone_saved_len: i32,
    audio_tone_len: i32,
    audio_tone: WIPICWord,          // MC_MdaToneType*
    audio_tone_duration: WIPICWord, // M_Int32 *

    audio_freq_saved_len: i32,
    audio_freq_len: i32,
    audio_hi_freq: WIPICWord,       // M_Int32 *
    audio_low_freq: WIPICWord,      // M_Int32 *
    audio_freq_duration: WIPICWord, // M_Int32 *

    sound_data_saved_len: i32,
    sound_data_len: i32,
    sound_data: WIPICWord, // M_Byte *

    original_volume: i32,

    pos: i8,
    _padding3: [u8; 3],
    codec_config_data_size: i32,
    codec_config_data: WIPICWord, // M_Byte *
    tick_duration: i32,

    b_control: u8, // bool
    _padding4: [u8; 3],

    movie_record_size_width: i32,
    movie_record_size_height: i32,
    max_record_length: i32,

    temp_record_space: WIPICWord, // M_Byte *
    temp_record_space_size: i32,
    temp_record_size: i32,

    next_ptr: WIPICWord, // MC_MdaClip*

    mda_id: i32,
    device_info: i32,

    // not in sdk, for internal usage
    handle: u32,
    /// 0..100, set by `MC_mdaClipSetVolume`; `clip_create` starts it at 100.
    volume: i32,
    /// Nonzero once `MC_mdaClipPutData` gave `handle` a sound — handle 0 is a real handle.
    loaded: u32,
    /// `MC_mdaClipCreate`'s `cb(clip, status)`; 0 = none.
    callback: WIPICWord,
    /// Bumped by every play and stop: an END_OF_DATA timer armed by an earlier play is void.
    generation: u32,
    /// Nonzero from a play until its END_OF_DATA or a stop.
    sounding: u32,
    /// Nonzero from a play until its STARTED callback returns.
    starting: u32,
}

impl MdaClip {
    fn apply_volume(&self, context: &mut dyn WIPICContext) {
        if self.loaded != 0 {
            let _ = context.system().audio().set_volume(self.handle, self.volume as f32 / 100.0);
        }
    }
}

static NEXT_CLIP_ID: AtomicI32 = AtomicI32::new(1);

pub async fn clip_create(context: &mut dyn WIPICContext, ptr_type: WIPICWord, buf_size: WIPICWord, callback: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_mdaClipCreate({ptr_type:#x}, {buf_size:#x}, {callback:#x})");

    let clip = context.alloc_raw(size_of::<MdaClip>() as u32)?;
    // The allocator does not zero, and `volume`/`loaded` are read before anything else writes them.
    // `clip_id` must be nonzero: games test it before playing. 9d52de42e7f9's play routine is
    // `if (!clip || !clip->clipId) return;` ahead of `MC_mdaPlay` (disassembled 2026-09-30), so with
    // the zeroed id it loaded all six sounds and never played one. A small counter, not the address:
    // an id is what a game may index a table with.
    write_generic(
        context,
        clip,
        MdaClip {
            clip_id: NEXT_CLIP_ID.fetch_add(1, Ordering::Relaxed),
            volume: 100,
            callback,
            ..Zeroable::zeroed()
        },
    )?;

    Ok(clip)
}

pub async fn clip_free(context: &mut dyn WIPICContext, clip: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_mdaClipFree({clip:#x})");

    // some app call clip free with null clip...
    if clip == 0 {
        return Ok(0);
    }

    // A zero id tells a pending callback the clip is gone (`ClipStatusCallback::live`).
    write_generic(context, clip, 0i32)?;
    context.free_raw(clip, size_of::<MdaClip>() as u32)?;

    Ok(0)
}

pub async fn clip_get_type(_context: &mut dyn WIPICContext, clip: WIPICWord, buf: WIPICWord, buf_size: WIPICWord) -> Result<WIPICWord> {
    tracing::warn!("stub MC_mdaClipGetType({clip:#x}, {buf:#x}, {buf_size:#x})");

    Ok(0)
}

pub async fn get_mute_state(_context: &mut dyn WIPICContext, source: WIPICWord) -> Result<WIPICWord> {
    tracing::warn!("stub MC_mdaGetMuteState({source:#x})");

    Ok(0)
}

pub async fn clip_get_info(
    _context: &mut dyn WIPICContext,
    clip: WIPICWord,
    command: WIPICWord,
    buf: WIPICWord,
    buf_size: WIPICWord,
) -> Result<WIPICWord> {
    tracing::warn!("stub OEMC_mdaClipGetInfo({clip:#x}, {command:#x}, {buf:#x}, {buf_size:#x})");

    Ok(0)
}

pub async fn clip_put_data(context: &mut dyn WIPICContext, ptr_clip: WIPICWord, buf: WIPICWord, buf_size: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_mdaClipPutData({ptr_clip:#x}, {buf:#x}, {buf_size:#x})");

    if ptr_clip == 0 {
        // -1 is outside the `WIPICError` vocabulary and stays: nothing interprets this value as
        // that enum. The `wipic_sys` wrapper for this call reads `if result < 0` and carries the
        // number through as `MediaError::Platform(i32)`, so the SDK's own contract here is
        // "negative means platform error" — which -1 satisfies. docs/wipi-c-abi-error-codes.md §4.
        return Ok(-1);
    }

    let mut data = vec![0; buf_size as _];
    context.read_bytes(buf, &mut data)?;

    let handle = context.system().audio().load_smaf(&data);
    if let Err(x) = handle {
        tracing::error!("Failed to load audio: {x:?}");
        return Ok(0);
    }

    let handle = handle.unwrap();

    let mut clip: MdaClip = read_generic(context, ptr_clip)?;
    clip.handle = handle;
    clip.loaded = 1;
    write_generic(context, ptr_clip, clip)?;
    clip.apply_volume(context);

    Ok(buf_size as _)
}

pub async fn clip_get_data(_context: &mut dyn WIPICContext, clip: WIPICWord, buf: WIPICWord, buf_size: WIPICWord) -> Result<WIPICWord> {
    tracing::warn!("stub MC_mdaClipGetData({clip:#x}, {buf:#x}, {buf_size:#x})");

    Ok(0)
}

pub async fn clip_set_position(_context: &mut dyn WIPICContext, clip: WIPICWord, ms: WIPICWord) -> Result<WIPICWord> {
    tracing::warn!("stub MC_mdaClipSetPosition({clip:#x}, {ms:#x})");

    Ok(0)
}

// Volumes are 0..100: measured 2026-09-27, 50 KTF/LGT titles call `MC_mdaClipSetVolume`, all within
// 0..100 (60, 50 and 40 the most common). 13 of them first read `MC_mdaClipGetVolume` and set what
// it answered — 0 while it was a stub, 100 now — so the getters must report the real volume before
// the setters may take effect, or those 13 go silent.
pub async fn clip_get_volume(context: &mut dyn WIPICContext, ptr_clip: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_mdaClipGetVolume({ptr_clip:#x})");

    if ptr_clip == 0 {
        return Ok(0);
    }
    let clip: MdaClip = read_generic(context, ptr_clip)?;

    Ok(clip.volume as _)
}

pub async fn clip_set_volume(context: &mut dyn WIPICContext, ptr_clip: WIPICWord, volume: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_mdaClipSetVolume({ptr_clip:#x}, {volume})");

    if ptr_clip == 0 {
        return Ok(0);
    }
    let mut clip: MdaClip = read_generic(context, ptr_clip)?;
    clip.volume = (volume as i32).clamp(0, 100);
    write_generic(context, ptr_clip, clip)?;
    clip.apply_volume(context);

    Ok(0)
}

pub async fn get_volume(context: &mut dyn WIPICContext) -> Result<WIPICWord> {
    tracing::debug!("MC_mdaGetVolume");

    Ok((context.system().audio().master_volume() * 100.0).round() as _)
}

pub async fn set_volume(context: &mut dyn WIPICContext, volume: i32) -> Result<()> {
    tracing::debug!("MC_mdaSetVolume({volume})");

    context.system().audio().set_master_volume(volume as f32 / 100.0);

    Ok(())
}

// Events follow the WIPI media model (AromaWIPI javadoc, `org.kwis.msp.media.Player`, the Java face
// of these calls): STARTED when a play begins, END_OF_DATA when its data runs out, STOPPED when a
// stop ends a play that was sounding — a stop on a stopped clip "does nothing". A repeat play's data
// never runs out, so nothing is sent at the loop boundary. Each event reaches `cb` after the call
// that caused it returns, like the `MC_netConnect` callbacks.
pub async fn play(context: &mut dyn WIPICContext, ptr_clip: WIPICWord, repeat: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_mdaPlay({ptr_clip:#x}, {repeat})");

    if ptr_clip == 0 {
        return Ok(0);
    }

    let mut clip: MdaClip = read_generic(context, ptr_clip)?;

    // "Playing a clip that is playing does nothing" (same javadoc). Held only while its STARTED is
    // pending, where a callback that answers STARTED with MC_mdaPlay would restart the song and
    // queue another STARTED forever; anywhere else a replay still restarts the sound, as it always did.
    if clip.starting != 0 && clip.sounding != 0 {
        tracing::debug!("MC_mdaPlay({ptr_clip:#x}): STARTED not yet delivered, ignored");
        return Ok(0);
    }

    if let Err(x) = context.system().audio().play(clip.handle, repeat != 0) {
        tracing::error!("Failed to load audio: {x:?}");
        return Ok(0);
    }

    clip.generation = clip.generation.wrapping_add(1);
    clip.sounding = 1;
    if clip.callback != 0 {
        clip.starting = 1;
    }
    write_generic(context, ptr_clip, clip)?;

    if clip.callback != 0 {
        // 87b04639cdfe (LGT, binary.mod 0x14334) takes the song's start time only in
        // `cb(clip, STARTED)` and places its notes against it, so without the event the song played
        // with no notes.
        context.spawn(Box::new(ClipStatusCallback::new(&clip, ptr_clip, MC_MDA_STATUS_STARTED)))?;

        // ponytail: a clip with no events (duration 0) never ends; END_OF_DATA at once would let a
        // callback that replays on END spin. Add it if a title waits on an empty clip.
        let duration = context.system().audio().duration(clip.handle).unwrap_or(0);
        if repeat == 0 && duration > 0 {
            let due = context.system().platform().now() + duration;
            context.set_timer(
                due,
                u64::MAX,
                Box::new(ClipStatusCallback::new(&clip, ptr_clip, MC_MDA_STATUS_END_OF_DATA)),
            );
        }
    }

    Ok(0)
}

const MC_MDA_STATUS_END_OF_DATA: WIPICWord = 1;
const MC_MDA_STATUS_STARTED: WIPICWord = 2;
const MC_MDA_STATUS_STOPPED: WIPICWord = 3;

struct ClipStatusCallback {
    callback: WIPICWord,
    clip: WIPICWord,
    clip_id: i32,
    generation: u32,
    status: WIPICWord,
}

impl ClipStatusCallback {
    fn new(clip: &MdaClip, ptr_clip: WIPICWord, status: WIPICWord) -> Self {
        Self {
            callback: clip.callback,
            clip: ptr_clip,
            clip_id: clip.clip_id,
            generation: clip.generation,
            status,
        }
    }

    /// The clip, unless it was freed since (`clip_free` zeroes the id; a new clip there has another).
    fn live(&self, context: &mut dyn WIPICContext) -> Result<Option<MdaClip>> {
        let clip: MdaClip = read_generic(context, self.clip)?;

        Ok((clip.clip_id == self.clip_id).then_some(clip))
    }
}

#[async_trait::async_trait]
impl MethodBody<WieError> for ClipStatusCallback {
    async fn call(&self, context: &mut dyn WIPICContext, _: Box<[WIPICWord]>) -> Result<WIPICResult> {
        let Some(mut clip) = self.live(context)? else {
            return Ok(WIPICResult { results: Vec::new() });
        };
        if self.status == MC_MDA_STATUS_END_OF_DATA {
            if clip.generation != self.generation {
                return Ok(WIPICResult { results: Vec::new() });
            }
            clip.sounding = 0;
            write_generic(context, self.clip, clip)?;
        }

        tracing::debug!("MC_mda clip callback {:#x}({:#x}, {})", self.callback, self.clip, self.status);
        context.call_function(self.callback, &[self.clip, self.status]).await?;

        if self.status == MC_MDA_STATUS_STARTED
            && let Some(mut clip) = self.live(context)?
        {
            clip.starting = 0;
            write_generic(context, self.clip, clip)?;
        }

        Ok(WIPICResult { results: Vec::new() })
    }
}

pub async fn clip_alloc_player(_context: &mut dyn WIPICContext, clip: WIPICWord, param: WIPICWord) -> Result<WIPICWord> {
    tracing::warn!("stub MC_mdaClipAllocPlayer({clip:#x}, {param:#x})");

    Ok(0)
}

pub async fn clip_free_player(_context: &mut dyn WIPICContext, clip: WIPICWord) -> Result<WIPICWord> {
    tracing::warn!("stub MC_mdaClipFreePlayer({clip:#x})");

    Ok(0)
}

pub async fn vibrator(context: &mut dyn WIPICContext, level: i32, timeout: i32) -> Result<WIPICWord> {
    tracing::debug!("MC_mdaVibrator({level}, {timeout})");

    let duration_ms = timeout.max(0) as u64;
    let intensity = (level.clamp(0, 10) * 10) as u8;
    context.system().platform().vibrate(duration_ms, intensity);

    Ok(0)
}

pub async fn set_mute_state(_context: &mut dyn WIPICContext, source: i32, b_mute: i32) -> Result<WIPICWord> {
    tracing::warn!("stub MC_mdaSetMuteState({source:#x}, {b_mute})");

    Ok(0)
}

pub async fn pause(_context: &mut dyn WIPICContext, clip: WIPICWord) -> Result<WIPICWord> {
    tracing::warn!("stub MC_mdaPause({clip:#x})");

    Ok(0)
}

pub async fn resume(_context: &mut dyn WIPICContext, clip: WIPICWord) -> Result<WIPICWord> {
    tracing::warn!("stub MC_mdaResume({clip:#x})");

    Ok(0)
}

pub async fn stop(context: &mut dyn WIPICContext, ptr_clip: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_mdaStop({ptr_clip:#x})");

    if ptr_clip == 0 {
        return Ok(0);
    }

    let mut clip: MdaClip = read_generic(context, ptr_clip)?;

    context.system().audio().stop(clip.handle);

    let was_sounding = clip.sounding != 0;
    clip.generation = clip.generation.wrapping_add(1);
    clip.sounding = 0;
    write_generic(context, ptr_clip, clip)?;
    if was_sounding && clip.callback != 0 {
        context.spawn(Box::new(ClipStatusCallback::new(&clip, ptr_clip, MC_MDA_STATUS_STOPPED)))?;
    }

    Ok(0)
}

pub async fn record(_context: &mut dyn WIPICContext, clip: WIPICWord) -> Result<WIPICWord> {
    tracing::warn!("stub MC_mdaRecord({clip:#x})");

    Ok(0)
}

pub async fn unk7(_context: &mut dyn WIPICContext, clip: WIPICWord) -> Result<WIPICWord> {
    tracing::warn!("stub MC_mdaUnk7({clip:#x})");

    Ok(0)
}

pub async fn unk17(_context: &mut dyn WIPICContext, clip: WIPICWord) -> Result<WIPICWord> {
    tracing::warn!("stub MC_mdaUnk17({clip:#x})");

    Ok(0)
}

pub async fn unk18(_context: &mut dyn WIPICContext, clip: WIPICWord) -> Result<WIPICWord> {
    tracing::warn!("stub MC_mdaUnk18({clip:#x})");

    Ok(0)
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, vec, vec::Vec};

    use test_utils::TestPlatform;
    use wie_backend::{AudioSequence, DefaultTaskRunner, System};

    use crate::context::{WIPICContext, test::TestContext};

    use wie_util::read_generic;

    use super::{MdaClip, clip_create, clip_free, clip_get_volume, clip_put_data, clip_set_volume, get_volume, play, set_volume, stop};

    // The getter answers 100 before any set: 13 titles read it and set what it said. A clip with no sound yet must not reach handle 0, which is the
    // first clip anybody loaded.
    #[futures_test::test]
    async fn clip_and_master_volume_reach_the_audio_handle_test() {
        let mut context = TestContext::with_system(System::new(Box::new(TestPlatform::new()), "pid", "aid", DefaultTaskRunner));
        let loaded = clip_create(&mut context, 0, 0, 0).await.unwrap();
        let empty = clip_create(&mut context, 0, 0, 0).await.unwrap();
        assert_eq!(clip_get_volume(&mut context, loaded).await.unwrap(), 100);

        clip_put_data(&mut context, loaded, 0x1000, 0).await.unwrap();
        clip_set_volume(&mut context, loaded, 40).await.unwrap();
        clip_set_volume(&mut context, empty, 10).await.unwrap();

        assert_eq!(clip_get_volume(&mut context, loaded).await.unwrap(), 40);
        assert_eq!(clip_get_volume(&mut context, empty).await.unwrap(), 10);
        assert_eq!(context.system().audio().volume(0), 0.4);

        assert_eq!(get_volume(&mut context).await.unwrap(), 100);
        set_volume(&mut context, 60).await.unwrap();
        assert_eq!(get_volume(&mut context).await.unwrap(), 60);
        assert_eq!(context.system().audio().master_volume(), 0.6);
    }

    // The first word of MC_MdaClip is its id, and games read it: 9d52de42e7f9 returns from its play
    // routine when `clipId == 0`, so a zeroed id loaded every sound and played none.
    #[futures_test::test]
    async fn created_clips_have_distinct_nonzero_ids_test() {
        let mut context = TestContext::with_system(System::new(Box::new(TestPlatform::new()), "pid", "aid", DefaultTaskRunner));
        let a = clip_create(&mut context, 0, 0, 0).await.unwrap();
        let b = clip_create(&mut context, 0, 0, 0).await.unwrap();
        let (a, b): (MdaClip, MdaClip) = (read_generic(&context, a).unwrap(), read_generic(&context, b).unwrap());

        assert_ne!(a.clip_id, 0);
        assert_ne!(b.clip_id, 0);
        assert_ne!(a.clip_id, b.clip_id);
    }

    fn context() -> TestContext {
        TestContext::with_system(System::new(Box::new(TestPlatform::new()), "pid", "aid", DefaultTaskRunner))
    }

    /// A clip with callback 0xcb whose sound lasts `duration` ms.
    async fn clip_lasting(context: &mut TestContext, duration: u64) -> u32 {
        let clip = clip_create(context, 0, 0, 0xcb).await.unwrap();
        clip_put_data(context, clip, 0x1000, 0).await.unwrap();
        let handle = read_generic::<MdaClip, _>(&*context, clip).unwrap().handle;
        let sequence = AudioSequence {
            duration,
            events: Vec::new(),
        };
        context.system().audio().replace(handle, sequence).unwrap();

        clip
    }

    async fn run_spawned(context: &mut TestContext) {
        for callback in core::mem::take(&mut context.spawned) {
            callback.call(context, Box::new([])).await.unwrap();
        }
    }

    async fn run_timers(context: &mut TestContext) {
        for (_, _, callback) in core::mem::take(&mut context.timers) {
            callback.call(context, Box::new([])).await.unwrap();
        }
    }

    fn statuses(context: &mut TestContext) -> Vec<u32> {
        core::mem::take(&mut context.call_args).into_iter().map(|(_, args)| args[1]).collect()
    }

    // 87b04639cdfe's clip callback (binary.mod 0x14334) stores the song's start time on
    // `cb(clip, MC_MDA_STATUS_STARTED)` and its notes fall against it: with no event the song played
    // and no note ever came down. A clip created without a callback gets nothing.
    #[futures_test::test]
    async fn play_tells_the_clip_callback_it_started_test() {
        let mut context = context();
        let with_cb = clip_create(&mut context, 0, 0, 0x14335).await.unwrap();
        let without_cb = clip_create(&mut context, 0, 0, 0).await.unwrap();
        clip_put_data(&mut context, with_cb, 0x1000, 0).await.unwrap();
        clip_put_data(&mut context, without_cb, 0x1000, 0).await.unwrap();

        play(&mut context, without_cb, 0).await.unwrap();
        assert!(context.spawned.is_empty());
        assert!(context.timers.is_empty());

        play(&mut context, with_cb, 0).await.unwrap();
        assert!(context.call_args.is_empty(), "the event comes after MC_mdaPlay returns");
        let callback = context.spawned.pop().unwrap();
        callback.call(&mut context, Box::new([])).await.unwrap();
        assert_eq!(context.call_args, [(0x14335, vec![with_cb, 2])]);
    }

    // A one-shot ends with END_OF_DATA when its length has elapsed, and is then stopped: a later
    // MC_mdaStop sends nothing.
    #[futures_test::test]
    async fn one_shot_ends_with_end_of_data_test() {
        let mut context = context();
        let clip = clip_lasting(&mut context, 500).await;
        let now = context.system().platform().now();

        play(&mut context, clip, 0).await.unwrap();
        run_spawned(&mut context).await;
        assert_eq!(context.timers.len(), 1);
        assert!((500..1000).contains(&(context.timers[0].0.raw() - now.raw())), "due when the sound ends");
        run_timers(&mut context).await;
        assert_eq!(context.call_args, [(0xcb, vec![clip, 2]), (0xcb, vec![clip, 1])]);

        context.call_args.clear();
        stop(&mut context, clip).await.unwrap();
        assert!(context.spawned.is_empty());
    }

    // MC_mdaStop on a sounding clip sends STOPPED after it returns and voids the play's END_OF_DATA;
    // a second stop sends nothing.
    #[futures_test::test]
    async fn stop_sends_stopped_and_voids_end_of_data_test() {
        let mut context = context();
        let clip = clip_lasting(&mut context, 500).await;

        play(&mut context, clip, 0).await.unwrap();
        run_spawned(&mut context).await;
        stop(&mut context, clip).await.unwrap();
        assert_eq!(statuses(&mut context), [2], "STOPPED comes after MC_mdaStop returns");
        run_spawned(&mut context).await;
        run_timers(&mut context).await;
        stop(&mut context, clip).await.unwrap();
        run_spawned(&mut context).await;

        assert_eq!(statuses(&mut context), [3]);
    }

    // A repeat play never runs out of data: no END_OF_DATA at the loop boundary, STOPPED on stop.
    // A replay voids the earlier play's END_OF_DATA.
    #[futures_test::test]
    async fn repeat_play_has_no_end_of_data_and_replay_voids_the_old_one_test() {
        let mut context = context();
        let clip = clip_lasting(&mut context, 500).await;

        play(&mut context, clip, 1).await.unwrap();
        run_spawned(&mut context).await;
        assert!(context.timers.is_empty());
        stop(&mut context, clip).await.unwrap();
        run_spawned(&mut context).await;
        assert_eq!(statuses(&mut context), [2, 3]);

        play(&mut context, clip, 0).await.unwrap();
        run_spawned(&mut context).await;
        play(&mut context, clip, 0).await.unwrap();
        run_spawned(&mut context).await;
        run_timers(&mut context).await;
        assert_eq!(statuses(&mut context), [2, 2, 1], "one END_OF_DATA, the second play's");
    }

    // A callback that answers STARTED with MC_mdaPlay on the same clip would restart the song and
    // queue another STARTED, forever. Playing a playing clip does nothing (AromaWIPI `Player.play`),
    // so that play is ignored; once STARTED has returned a replay restarts the sound as before.
    #[futures_test::test]
    async fn play_inside_its_own_started_is_ignored_test() {
        let mut context = context();
        let clip = clip_lasting(&mut context, 500).await;

        play(&mut context, clip, 0).await.unwrap();
        // what the callback does, while STARTED is being delivered
        play(&mut context, clip, 0).await.unwrap();
        assert_eq!(context.spawned.len(), 1);
        assert_eq!(context.timers.len(), 1);
        run_spawned(&mut context).await;

        play(&mut context, clip, 0).await.unwrap();
        assert_eq!(context.spawned.len(), 1, "after STARTED a replay starts again");
    }

    // A stop inside STARTED ends the play, so a play after it starts again — the guard is "already
    // playing", not "inside a callback".
    #[futures_test::test]
    async fn play_after_stop_inside_started_starts_again_test() {
        let mut context = context();
        let clip = clip_lasting(&mut context, 500).await;

        play(&mut context, clip, 0).await.unwrap();
        stop(&mut context, clip).await.unwrap();
        play(&mut context, clip, 0).await.unwrap();

        assert_eq!(context.spawned.len(), 3);
    }

    // A freed clip hears nothing more: MC_mdaPlay then MC_mdaClipFree leaves STARTED and
    // END_OF_DATA pending against memory that is no longer a clip.
    #[futures_test::test]
    async fn freed_clip_gets_no_callback_test() {
        let mut context = context();
        let clip = clip_lasting(&mut context, 500).await;

        play(&mut context, clip, 0).await.unwrap();
        clip_free(&mut context, clip).await.unwrap();
        run_spawned(&mut context).await;
        run_timers(&mut context).await;

        assert!(context.call_args.is_empty());
    }
}
