use alloc::vec;

use bytemuck::{Pod, Zeroable};

use wipi_types::wipic::WIPICWord;

use wie_util::{Result, read_generic, write_generic};

use crate::context::WIPICContext;

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
}

impl MdaClip {
    fn apply_volume(&self, context: &mut dyn WIPICContext) {
        if self.loaded != 0 {
            let _ = context.system().audio().set_volume(self.handle, self.volume as f32 / 100.0);
        }
    }
}

pub async fn clip_create(context: &mut dyn WIPICContext, ptr_type: WIPICWord, buf_size: WIPICWord, callback: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_mdaClipCreate({ptr_type:#x}, {buf_size:#x}, {callback:#x})");

    let clip = context.alloc_raw(size_of::<MdaClip>() as u32)?;
    // The allocator does not zero, and `volume`/`loaded` are read before anything else writes them.
    // `clip_id` must be nonzero: games test it before playing. 9d52de42e7f9's play routine is
    // `if (!clip || !clip->clipId) return;` ahead of `MC_mdaPlay` (disassembled 2026-09-30), so with
    // the zeroed id it loaded all six sounds and never played one. The address is unique per live clip.
    write_generic(
        context,
        clip,
        MdaClip {
            clip_id: clip as i32,
            volume: 100,
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

pub async fn play(context: &mut dyn WIPICContext, ptr_clip: WIPICWord, repeat: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_mdaPlay({ptr_clip:#x}, {repeat})");

    if ptr_clip == 0 {
        return Ok(0);
    }

    let clip: MdaClip = read_generic(context, ptr_clip)?;

    let result = context.system().audio().play(clip.handle, repeat != 0);

    if let Err(x) = result {
        tracing::error!("Failed to load audio: {x:?}");
    }

    Ok(0)
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

    let clip: MdaClip = read_generic(context, ptr_clip)?;

    let system = context.system();

    system.audio().stop(clip.handle);

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
    use alloc::boxed::Box;

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};

    use crate::context::{WIPICContext, test::TestContext};

    use wie_util::read_generic;

    use super::{MdaClip, clip_create, clip_get_volume, clip_put_data, clip_set_volume, get_volume, set_volume};

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
}
