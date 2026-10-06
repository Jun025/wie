use alloc::{vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, JavaError, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class net.wie.WieAudioClip
//
// Every clip SKT titles hand this class is SMAF: measured 2026-09-27 over 5 SKVM titles (더팜1,
// 웰루시아, 고래사냥2, 아슬아슬타워쿤, 노리타이쿤), all 41 `open` payloads start with `MMMD` and
// `AudioSystem.getAudioClip` is always called with "mmf". So the clip rides the same
// `Audio::load_smaf` path WIPI-C `MC_mdaClipPutData` and MIDP's `SmafPlayer` use. Until then
// every method was a stub and SKT titles sent the audio sink nothing at all.
//
// One clip is opened many times (더팜1: one `<init>`, 20 `open`s), so `open` releases the
// previous handle. `audioHandle` stores handle + 1 so that 0 means "nothing loaded" — handle 0 is
// a real handle. `pause`/`resume` have no position to keep (the sink has no seek): `pause` stops
// and `resume` restarts from the top with the mode the clip was last started in.
//
// `close` does NOT stop the sound. 더팜1 plays every effect as `open` → `play` → `close`, and
// stopping there made all of them silent (measured in the browser: each `play` followed by a
// `stop` in the same instant, output RMS 0). So `close` only retires the clip — later `play`s
// are ignored — and the handle is released by the next `open`, which also bounds what stays
// loaded to one handle per clip. `stop`/`pause` on a closed clip are ignored too: 드래곤나이트EX
// calls `play` → `close` → `stop` within 66 µs (measured 2026-09-27), so a `stop` that reached a
// closed clip's sound would cut its effects the way stopping in `close` cut 더팜1's.
//
// That leaves a loop whose clip was closed with nothing that can stop it except `open` on that
// same object. 사고뭉치트윈스 starts its music on one clip (`open` → `loop` → `close`), calls
// `stop` on it (ignored, closed), then plays the next song from a NEW clip — and the first song
// kept looping under the second (measured: `loop_overlaps` 1, the only one in 50 SKT titles).
// So a loop start stops every loop left behind by a closed clip (`orphanLoops`). Loops of clips
// that are still open are left alone — the game can still stop those itself.
//
// `loop` on a clip that is already looping does nothing: 나이트메이커 calls it every frame, and
// restarting there sent 86,769 plays in 60 s, each cutting the music back to its first note.
pub struct WieAudioClip;

impl WieAudioClip {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/WieAudioClip",
            parent_class: Some("java/lang/Object"),
            interfaces: vec!["com/skt/m/AudioClip"],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("open", "([BII)V", Self::open, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("close", "()V", Self::close, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("loop", "()V", Self::r#loop, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("pause", "()V", Self::pause, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("play", "()V", Self::play, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("resume", "()V", Self::resume, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("stop", "()V", Self::stop, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("audioHandle", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("repeat", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("paused", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("closed", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("looping", "Z", FieldAccessFlags::PRIVATE),
                // Host ms at which the last one-shot play() runs out; 0 = none sounding.
                JavaFieldProto::new("soundingUntil", "J", FieldAccessFlags::PRIVATE),
                // A play() on a sound thread is waiting for this clip to run out; and stop()/close()
                // ended that wait (see `play`).
                JavaFieldProto::new("blocking", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("userStopped", "Z", FieldAccessFlags::PRIVATE),
                // Handles (+ 1, like `audioHandle`) of loops whose clip was closed mid-loop.
                JavaFieldProto::new("orphanLoops", "[I", FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::<init>({this:?}, {name:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn open(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        buffer_size: i32,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::open({this:?}, {data:?}, {offset}, {buffer_size})");

        if data.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "data is null").await);
        }

        let array_length = jvm.array_length(&data).await?;
        if offset < 0 || buffer_size < 0 || (offset as usize).checked_add(buffer_size as usize).is_none_or(|end| end > array_length) {
            return Err(jvm
                .exception("java/lang/ArrayIndexOutOfBoundsException", "Invalid offset or length")
                .await);
        }

        Self::release(jvm, context, &mut this).await?;
        jvm.put_field(&mut this, "closed", "Z", false).await?;

        let bytes: Vec<i8> = jvm.load_array(&data, offset as usize, buffer_size as usize).await?;
        let bytes: Vec<u8> = bytes.into_iter().map(|byte| byte as u8).collect();
        match context.system().audio().load_smaf(&bytes) {
            Ok(handle) => jvm.put_field(&mut this, "audioHandle", "I", handle as i32 + 1).await?,
            Err(error) => tracing::error!("net.wie.WieAudioClip::open: failed to load audio: {error:?}"),
        }

        Ok(())
    }

    /// The loaded handle, or `None` if nothing is loaded or the clip was closed.
    async fn handle(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Option<u32>> {
        let closed: bool = jvm.get_field(this, "closed", "Z").await?;
        if closed {
            return Ok(None);
        }

        Self::loaded(jvm, this).await
    }

    async fn loaded(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Option<u32>> {
        let stored: i32 = jvm.get_field(this, "audioHandle", "I").await?;

        Ok((stored > 0).then(|| stored as u32 - 1))
    }

    async fn start(jvm: &Jvm, context: &mut WieJvmContext, this: &mut ClassInstanceRef<Self>, repeat: bool) -> JvmResult<()> {
        let Some(handle) = Self::handle(jvm, this).await? else {
            jvm.put_field(this, "repeat", "Z", repeat).await?;
            jvm.put_field(this, "paused", "Z", false).await?;
            return Ok(());
        };
        let looping: bool = jvm.get_field(this, "looping", "Z").await?;
        if repeat && looping {
            return Ok(());
        }
        let now = context.system().platform().now().raw() as i64;
        let sounding_until: i64 = jvm.get_field(this, "soundingUntil", "J").await?;
        if !repeat && now < sounding_until {
            return Ok(());
        }

        jvm.put_field(this, "repeat", "Z", repeat).await?;
        jvm.put_field(this, "paused", "Z", false).await?;
        jvm.put_field(this, "looping", "Z", repeat).await?;
        if repeat {
            Self::stop_orphan_loops(jvm, context).await?;
        }
        if let Err(error) = context.system().audio().play(handle, repeat) {
            tracing::error!("net.wie.WieAudioClip: failed to play audio: {error:?}");
        }
        let duration = if repeat { None } else { context.system().audio().duration(handle) };
        let until = duration.map_or(0, |duration| now + duration as i64);
        jvm.put_field(this, "soundingUntil", "J", until).await?;

        Ok(())
    }

    async fn orphan_loops(jvm: &Jvm) -> JvmResult<Vec<i32>> {
        let orphans: ClassInstanceRef<Array<i32>> = jvm.get_static_field("net/wie/WieAudioClip", "orphanLoops", "[I").await?;
        if orphans.is_null() {
            return Ok(Vec::new());
        }
        let length = jvm.array_length(&orphans).await?;

        jvm.load_array(&orphans, 0, length).await
    }

    async fn set_orphan_loops(jvm: &Jvm, handles: &[i32]) -> JvmResult<()> {
        let mut orphans = jvm.instantiate_array("I", handles.len()).await?;
        jvm.store_array(&mut orphans, 0, handles.iter().copied()).await?;

        jvm.put_static_field("net/wie/WieAudioClip", "orphanLoops", "[I", orphans).await
    }

    /// Stops every loop left behind by a closed clip. A stale entry (its handle since released by
    /// `open`) is harmless: stopping a handle that is not playing sends nothing.
    async fn stop_orphan_loops(jvm: &Jvm, context: &mut WieJvmContext) -> JvmResult<()> {
        let orphans = Self::orphan_loops(jvm).await?;
        if orphans.is_empty() {
            return Ok(());
        }
        for stored in orphans {
            context.system().audio().stop(stored as u32 - 1);
        }

        Self::set_orphan_loops(jvm, &[]).await
    }

    async fn release(jvm: &Jvm, context: &mut WieJvmContext, this: &mut ClassInstanceRef<Self>) -> JvmResult<()> {
        if let Some(handle) = Self::loaded(jvm, this).await? {
            let _ = context.system().audio().close(handle);
            jvm.put_field(this, "audioHandle", "I", 0).await?;
        }
        jvm.put_field(this, "paused", "Z", false).await?;
        jvm.put_field(this, "looping", "Z", false).await?;
        jvm.put_field(this, "soundingUntil", "J", 0i64).await?;

        Ok(())
    }

    // Ends with a yield. f6fe2adc8cce runs its music from a thread whose whole body is
    // `while (a) clip.play();` — no sleep, no wait. On the handset that thread is preempted; here
    // guest threads are cooperative, so a play() that returns at once kept that one loop running
    // forever and the tick it was in never ended (measured 2026-09-27: 150 s on one tick, the
    // interpreter trace a two-instruction loop around this call).
    //
    // And a play() on a clip whose last play() is still sounding is ignored (`soundingUntil`, set
    // from the sequence's length). With the yield alone that loop restarted its song 146,619 times
    // in a 30 s probe; ignoring it plays the song through and restarts it once it ends, which is
    // what the loop is for. `loop` on a looping clip is ignored for the same reason (header).
    // The cost: a game that re-triggers one clip faster than it lasts no longer cuts it short.
    //
    // On a sound thread play() blocks until the clip runs out, and a stop() or close() from another
    // thread ends it with `UserStopException` — the handset's behaviour, which the SKT sound threads
    // are written against (docs/report/0463). 71d1d8235bd1's thread loops
    // `synchronized (this) { while (!isPlaying) { if (isRepeat) sleep(100); else wait(); } }` and then
    // plays outside the lock; its stop() is `clip.close()`, and only the exception out of play()
    // clears `isRepeat`. A play() that returned at once left the thread re-opening the song every
    // 100 ms, and a stop() landing between two of those left it asleep holding the lock for good.
    // 47fe675bfffd catches `UserStopException` around its play(); every other one of the 94 play()
    // sites in the SKT corpus catches `Exception`.
    //
    // "A sound thread" is read off the Java stack: `run()` is the caller or the caller's caller.
    // Every one of those 94 sites is in a `run()` that only plays sound, or in a helper that such a
    // `run()` calls; an effect played from the game's own code (1367261bc3ee: an event handler ->
    // helper -> play) is deeper and does not block, so it cannot freeze the game.
    // ponytail: depth, not "what this thread is for" — a game loop whose run() called a sound helper
    // directly would block on its effects; the corpus has none (docs/report/0463 §1).
    async fn play(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::play({this:?})");

        Self::start(jvm, context, &mut this, false).await?;
        if Self::on_sound_thread(jvm) && Self::handle(jvm, &this).await?.is_some() {
            Self::wait_until_done(jvm, context, &mut this).await?;
        }
        jvm.invoke_static("java/lang/Thread", "yield", "()V", ()).await
    }

    fn on_sound_thread(jvm: &Jvm) -> bool {
        // [0] is this play() itself.
        jvm.stack_trace().iter().skip(1).take(2).any(|frame| frame.ends_with(".run()V"))
    }

    async fn wait_until_done(jvm: &Jvm, context: &mut WieJvmContext, this: &mut ClassInstanceRef<Self>) -> JvmResult<()> {
        jvm.put_field(this, "blocking", "Z", true).await?;
        jvm.put_field(this, "userStopped", "Z", false).await?;
        let result = async {
            loop {
                if jvm.get_field(this, "userStopped", "Z").await? {
                    let exception = jvm.new_class("com/skt/m/UserStopException", "()V", ()).await?;
                    return Err(JavaError::JavaException(exception));
                }
                let paused: bool = jvm.get_field(this, "paused", "Z").await?;
                let until: i64 = jvm.get_field(this, "soundingUntil", "J").await?;
                let now = context.system().platform().now().raw() as i64;
                if !paused && now >= until {
                    return Ok(());
                }
                let step = if paused { 20 } else { (until - now).min(20) };
                let _: () = jvm.invoke_static("java/lang/Thread", "sleep", "(J)V", (step,)).await?;
            }
        }
        .await;
        jvm.put_field(this, "blocking", "Z", false).await?;

        result
    }

    /// stop()/close() from another thread while a sound thread's play() waits on this clip.
    async fn stop_user(jvm: &Jvm, this: &mut ClassInstanceRef<Self>) -> JvmResult<bool> {
        let blocking: bool = jvm.get_field(this, "blocking", "Z").await?;
        if blocking {
            jvm.put_field(this, "userStopped", "Z", true).await?;
        }

        Ok(blocking)
    }

    async fn r#loop(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::loop({this:?})");

        Self::start(jvm, context, &mut this, true).await
    }

    async fn pause(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::pause({this:?})");

        if let Some(handle) = Self::handle(jvm, &this).await? {
            context.system().audio().stop(handle);
            jvm.put_field(&mut this, "paused", "Z", true).await?;
            jvm.put_field(&mut this, "looping", "Z", false).await?;
            jvm.put_field(&mut this, "soundingUntil", "J", 0i64).await?;
        }

        Ok(())
    }

    async fn resume(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::resume({this:?})");

        let paused: bool = jvm.get_field(&this, "paused", "Z").await?;
        if paused {
            let repeat: bool = jvm.get_field(&this, "repeat", "Z").await?;
            Self::start(jvm, context, &mut this, repeat).await?;
        }

        Ok(())
    }

    async fn stop(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::stop({this:?})");

        Self::stop_user(jvm, &mut this).await?;
        if let Some(handle) = Self::handle(jvm, &this).await? {
            context.system().audio().stop(handle);
            jvm.put_field(&mut this, "looping", "Z", false).await?;
        }
        jvm.put_field(&mut this, "paused", "Z", false).await?;
        jvm.put_field(&mut this, "soundingUntil", "J", 0i64).await?;

        Ok(())
    }

    async fn close(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::close({this:?})");

        let closed: bool = jvm.get_field(&this, "closed", "Z").await?;
        // Ending a sound thread's play() ends its sound too (header: this is not 더팜1's close).
        if !closed
            && Self::stop_user(jvm, &mut this).await?
            && let Some(handle) = Self::loaded(jvm, &this).await?
        {
            context.system().audio().stop(handle);
        }
        let looping: bool = jvm.get_field(&this, "looping", "Z").await?;
        if !closed
            && looping
            && let Some(handle) = Self::loaded(jvm, &this).await?
        {
            let mut orphans = Self::orphan_loops(jvm).await?;
            orphans.push(handle as i32 + 1);
            Self::set_orphan_loops(jvm, &orphans).await?;
        }
        jvm.put_field(&mut this, "closed", "Z", true).await?;
        jvm.put_field(&mut this, "paused", "Z", false).await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, sync::Arc, vec, vec::Vec};
    use core::sync::atomic::{AtomicBool, Ordering};

    use jvm::{Array, ClassInstanceRef, JavaError, Result as JvmResult, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::{TestClock, TestPlatform, run_jvm_test, run_jvm_test_with_system};
    use wie_backend::AudioCommand;

    use jvm::Jvm;
    use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

    use crate::{classes::com::skt::m::AudioClip, get_protos};

    /// The shape of an SKT sound thread: `run()` plays the static clip, directly or (`viaHelper`)
    /// through a helper as 8 titles do; `effect()` plays it through the same helper from anywhere
    /// else. `outcome`: 1 = play() returned, 2 = it threw `UserStopException`.
    struct SoundThread;

    impl SoundThread {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/SoundThread",
                parent_class: Some("java/lang/Object"),
                interfaces: vec!["java/lang/Runnable"],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("run", "()V", Self::run, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("effect", "()V", Self::effect, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("helper", "()V", Self::helper, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                ],
                fields: vec![
                    JavaFieldProto::new("clip", "Lcom/skt/m/AudioClip;", FieldAccessFlags::STATIC),
                    JavaFieldProto::new("outcome", "I", FieldAccessFlags::STATIC),
                    JavaFieldProto::new("viaHelper", "Z", FieldAccessFlags::STATIC),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
        }

        async fn run(jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<()> {
            let clip: ClassInstanceRef<AudioClip> = jvm.get_static_field("test/SoundThread", "clip", "Lcom/skt/m/AudioClip;").await?;
            let via_helper: bool = jvm.get_static_field("test/SoundThread", "viaHelper", "Z").await?;
            let played = if via_helper {
                jvm.invoke_static::<_, ()>("test/SoundThread", "helper", "()V", ()).await
            } else {
                jvm.invoke_virtual::<_, ()>(&clip, "com/skt/m/AudioClip", "play", "()V", ()).await
            };
            let outcome = match played {
                Ok(()) => 1,
                Err(JavaError::JavaException(exception)) if jvm.is_instance(&*exception, "com/skt/m/UserStopException") => 2,
                Err(error) => return Err(error),
            };
            jvm.put_static_field("test/SoundThread", "outcome", "I", outcome).await
        }

        async fn effect(jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_static("test/SoundThread", "helper", "()V", ()).await
        }

        async fn helper(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
            let clip: ClassInstanceRef<AudioClip> = jvm.get_static_field("test/SoundThread", "clip", "Lcom/skt/m/AudioClip;").await?;
            jvm.invoke_virtual(&clip, "com/skt/m/AudioClip", "play", "()V", ()).await
        }
    }

    /// A minimal SMAF: one `SEQU` chunk, two notes 50 × 20 ms apart, then end of stream.
    fn smaf() -> Vec<u8> {
        let sequence = [0x00, 0x01, 0x0a, 0x32, 0x01, 0x0a, 0, 0, 0, 0];
        let mut smaf = b"MMMD\0\0\0\0SEQU".to_vec();
        smaf.extend_from_slice(&(sequence.len() as u32).to_be_bytes());
        smaf.extend_from_slice(&sequence);
        smaf.extend_from_slice(&[0, 0]);
        smaf
    }

    /// Opens `smaf()` on a new clip and stores it in `SoundThread.clip`; returns it and a `SoundThread`.
    async fn open_sound_thread_clip(jvm: &Jvm) -> JvmResult<(ClassInstanceRef<AudioClip>, Box<dyn jvm::ClassInstance>)> {
        let name = JavaLangString::from_rust_string(jvm, "mmf").await?;
        let clip: ClassInstanceRef<AudioClip> = jvm
            .invoke_static(
                "com/skt/m/AudioSystem",
                "getAudioClip",
                "(Ljava/lang/String;)Lcom/skt/m/AudioClip;",
                (name,),
            )
            .await?;
        let bytes = smaf();
        let mut data = jvm.instantiate_array("B", bytes.len()).await?;
        jvm.store_array(&mut data, 0, bytes.iter().map(|&x| x as i8)).await?;
        let _: () = jvm
            .invoke_virtual(&clip, "net/wie/WieAudioClip", "open", "([BII)V", (data, 0, bytes.len() as i32))
            .await?;
        jvm.put_static_field("test/SoundThread", "clip", "Lcom/skt/m/AudioClip;", clip.clone())
            .await?;
        let thread = jvm.new_class("test/SoundThread", "()V", ()).await?;

        Ok((clip, thread))
    }

    #[test]
    fn audio_clip_accepts_valid_slices_and_rejects_invalid_ranges() {
        let result = run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let formats: ClassInstanceRef<Array<String>> = jvm
                .invoke_static("com/skt/m/AudioSystem", "getClipFormats", "()[Ljava/lang/String;", ())
                .await?;
            assert_eq!(jvm.array_length(&formats).await?, 0);

            let name = JavaLangString::from_rust_string(&jvm, "audio/mpeg").await?;
            let clip: ClassInstanceRef<AudioClip> = jvm
                .invoke_static(
                    "com/skt/m/AudioSystem",
                    "getAudioClip",
                    "(Ljava/lang/String;)Lcom/skt/m/AudioClip;",
                    (name,),
                )
                .await?;
            let mut data = jvm.instantiate_array("B", 3).await?;
            jvm.store_array(&mut data, 0, [1i8, 2, 3]).await?;
            let empty_data = jvm.instantiate_array("B", 0).await?;

            for (array, offset, length) in [
                (empty_data, 0, 0),
                (data.clone(), 0, 0),
                (data.clone(), 3, 0),
                (data.clone(), 0, 3),
                (data.clone(), 1, 2),
            ] {
                let _: () = jvm
                    .invoke_virtual(&clip, "net/wie/WieAudioClip", "open", "([BII)V", (array, offset, length))
                    .await?;
            }
            let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "play", "()V", ()).await?;
            let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "loop", "()V", ()).await?;
            let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "pause", "()V", ()).await?;
            let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "resume", "()V", ()).await?;
            let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "stop", "()V", ()).await?;
            let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "close", "()V", ()).await?;

            for (offset, length) in [(-1, 1), (0, -1), (3, 1), (2, 2)] {
                let invalid: JvmResult<()> = jvm
                    .invoke_virtual(&clip, "net/wie/WieAudioClip", "open", "([BII)V", (data.clone(), offset, length))
                    .await;
                let Err(JavaError::JavaException(exception)) = invalid else {
                    panic!("AudioClip.open accepted invalid range ({offset}, {length})");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/ArrayIndexOutOfBoundsException"));
            }

            let null_data = ClassInstanceRef::<Array<i8>>::new(None);
            let null_result: JvmResult<()> = jvm
                .invoke_virtual(&clip, "net/wie/WieAudioClip", "open", "([BII)V", (null_data, 0, 0))
                .await;
            let Err(JavaError::JavaException(exception)) = null_result else {
                panic!("AudioClip.open accepted a null byte array");
            };
            assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));

            Ok(())
        });

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }

    /// The clip reaches the audio sink: before 2026-09-27 every method here was a stub, so
    /// SKT titles sent nothing and were silent on every host. Any of these reverting to a stub
    /// drops a command from the log below.
    #[test]
    fn audio_clip_drives_the_audio_sink() {
        let platform = TestPlatform::new();
        let log = platform.audio_log();

        let result = run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), get_protos().into()]),
            Box::new(platform),
            |jvm, _system| async move {
                let name = JavaLangString::from_rust_string(&jvm, "mmf").await?;
                let clip: ClassInstanceRef<AudioClip> = jvm
                    .invoke_static(
                        "com/skt/m/AudioSystem",
                        "getAudioClip",
                        "(Ljava/lang/String;)Lcom/skt/m/AudioClip;",
                        (name,),
                    )
                    .await?;
                let data = jvm.instantiate_array("B", 4).await?;

                // Nothing loaded yet: must not reach the sink (and must not fail).
                let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "play", "()V", ()).await?;
                let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "close", "()V", ()).await?;

                let _: () = jvm
                    .invoke_virtual(&clip, "net/wie/WieAudioClip", "open", "([BII)V", (data.clone(), 0, 4))
                    .await?;
                let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "loop", "()V", ()).await?;
                let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "pause", "()V", ()).await?;
                let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "resume", "()V", ()).await?;
                let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "stop", "()V", ()).await?;
                let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "play", "()V", ()).await?;
                // 더팜1's pattern: close right after play must leave the sound playing, and a
                // closed clip ignores play. The next open releases (stops) the old handle.
                let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "close", "()V", ()).await?;
                let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "play", "()V", ()).await?;
                let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "open", "([BII)V", (data, 0, 4)).await?;
                let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "play", "()V", ()).await?;
                let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "close", "()V", ()).await?;
                let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "close", "()V", ()).await?;

                Ok(())
            },
        );
        assert!(result.is_ok(), "JVM test failed: {result:?}");

        let shape: Vec<(&str, u32, bool)> = log
            .lock()
            .iter()
            .map(|command| match command {
                AudioCommand::Play { handle, repeat, .. } => ("play", *handle, *repeat),
                AudioCommand::Stop { handle } => ("stop", *handle, false),
            })
            .collect();
        assert_eq!(
            shape,
            [
                ("play", 0, true),  // loop()
                ("stop", 0, false), // pause()
                ("play", 0, true),  // resume() keeps the loop mode
                ("stop", 0, false), // stop()
                ("play", 0, false), // play() — then close() and a closed play() send nothing
                ("stop", 0, false), // open() again releases the first handle
                ("play", 1, false), // and the close()s after it send nothing either
            ]
        );
    }

    /// 사고뭉치트윈스's song change, and 나이트메이커's per-frame `loop`. A loop left behind by a
    /// closed clip is stopped when another loop starts; an open clip's loop is not (the game can
    /// still stop it); and `loop` on a clip that is already looping sends nothing.
    #[test]
    fn audio_clip_loop_stops_loops_orphaned_by_close() {
        let platform = TestPlatform::new();
        let log = platform.audio_log();

        let result = run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), get_protos().into()]),
            Box::new(platform),
            |jvm, _system| async move {
                let mut clips: Vec<ClassInstanceRef<AudioClip>> = Vec::new();
                for _ in 0..3 {
                    let name = JavaLangString::from_rust_string(&jvm, "mmf").await?;
                    clips.push(
                        jvm.invoke_static(
                            "com/skt/m/AudioSystem",
                            "getAudioClip",
                            "(Ljava/lang/String;)Lcom/skt/m/AudioClip;",
                            (name,),
                        )
                        .await?,
                    );
                }
                let data = jvm.instantiate_array("B", 4).await?;
                let call = async |clip: &ClassInstanceRef<AudioClip>, method: &str| -> JvmResult<()> {
                    jvm.invoke_virtual(clip, "net/wie/WieAudioClip", method, "()V", ()).await
                };

                // Song 1 on clip 0, exactly as 사고뭉치트윈스 does it.
                let _: () = jvm
                    .invoke_virtual(&clips[0], "net/wie/WieAudioClip", "open", "([BII)V", (data.clone(), 0, 4))
                    .await?;
                call(&clips[0], "loop").await?;
                call(&clips[0], "loop").await?; // already looping: nothing
                call(&clips[0], "close").await?;
                call(&clips[0], "stop").await?; // closed: ignored
                // Song 2 on a new clip: song 1 must stop first.
                let _: () = jvm
                    .invoke_virtual(&clips[1], "net/wie/WieAudioClip", "open", "([BII)V", (data.clone(), 0, 4))
                    .await?;
                call(&clips[1], "loop").await?;
                // A loop on a third clip while clip 1 is still open: clip 1's loop is left alone.
                let _: () = jvm
                    .invoke_virtual(&clips[2], "net/wie/WieAudioClip", "open", "([BII)V", (data, 0, 4))
                    .await?;
                call(&clips[2], "loop").await?;

                Ok(())
            },
        );
        assert!(result.is_ok(), "JVM test failed: {result:?}");

        let shape: Vec<(&str, u32, bool)> = log
            .lock()
            .iter()
            .map(|command| match command {
                AudioCommand::Play { handle, repeat, .. } => ("play", *handle, *repeat),
                AudioCommand::Stop { handle } => ("stop", *handle, false),
            })
            .collect();
        assert_eq!(shape, [("play", 0, true), ("stop", 0, false), ("play", 1, true), ("play", 2, true)]);
    }

    /// `play()` hands the thread over. f6fe2adc8cce's music thread is `while (a) clip.play();`;
    /// without the yield another task never runs and the tick never ends. Here the other task is
    /// spawned first and must have run by the time the loop's second `play()` returns.
    #[test]
    fn play_yields_so_a_play_loop_does_not_starve_other_threads() {
        let result = run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), get_protos().into()]),
            Box::new(TestPlatform::new()),
            |jvm, system| async move {
                let ran = Arc::new(AtomicBool::new(false));
                let ran_clone = ran.clone();
                system.spawn(async move || {
                    ran_clone.store(true, Ordering::SeqCst);
                    Ok(())
                });

                let name = JavaLangString::from_rust_string(&jvm, "mmf").await?;
                let clip: ClassInstanceRef<AudioClip> = jvm
                    .invoke_static(
                        "com/skt/m/AudioSystem",
                        "getAudioClip",
                        "(Ljava/lang/String;)Lcom/skt/m/AudioClip;",
                        (name,),
                    )
                    .await?;
                // Two, as in the guest's loop: a task spawned during a step is first polled in the
                // next one, after this lower-numbered task has resumed.
                for _ in 0..2 {
                    let _: () = jvm.invoke_virtual(&clip, "com/skt/m/AudioClip", "play", "()V", ()).await?;
                }
                assert!(ran.load(Ordering::SeqCst), "play() returned without letting the other thread run");

                Ok(())
            },
        );
        assert!(result.is_ok(), "{result:?}");
    }

    /// A clip replayed while its last play() is still sounding is not restarted — f6fe2adc8cce's
    /// `while (a) clip.play();` sent 146,619 Play commands in a 30 s probe before this. Once the
    /// song has run out (the clock passes its length) or the clip was stopped, play() starts it again.
    #[test]
    fn play_is_not_restarted_while_the_clip_is_still_sounding() {
        let smaf = smaf();
        let clock = TestClock::new();
        let platform = TestPlatform::with_clock(clock.clone());
        let log = platform.audio_log();

        let result = run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), get_protos().into()]),
            Box::new(platform),
            move |jvm, system| async move {
                let name = JavaLangString::from_rust_string(&jvm, "mmf").await?;
                let clip: ClassInstanceRef<AudioClip> = jvm
                    .invoke_static(
                        "com/skt/m/AudioSystem",
                        "getAudioClip",
                        "(Ljava/lang/String;)Lcom/skt/m/AudioClip;",
                        (name,),
                    )
                    .await?;
                let mut data = jvm.instantiate_array("B", smaf.len()).await?;
                jvm.store_array(&mut data, 0, smaf.iter().map(|&x| x as i8)).await?;
                let _: () = jvm
                    .invoke_virtual(&clip, "net/wie/WieAudioClip", "open", "([BII)V", (data, 0, smaf.len() as i32))
                    .await?;
                let length = system.audio().duration(0).unwrap();
                assert!(length >= 1000, "{length}");

                let play = async || jvm.invoke_virtual::<_, ()>(&clip, "net/wie/WieAudioClip", "play", "()V", ()).await;
                play().await?; // plays
                play().await?; // still sounding: ignored
                clock.advance(length);
                play().await?; // ran out: plays again
                let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "stop", "()V", ()).await?;
                play().await?; // stopped: plays again at once
                clock.advance(1000); // past the tick budget, so the tick this ran in can end

                Ok(())
            },
        );
        assert!(result.is_ok(), "{result:?}");

        let plays = log.lock().iter().filter(|command| matches!(command, AudioCommand::Play { .. })).count();
        assert_eq!(plays, 3);
    }

    /// 71d1d8235bd1's sound thread: play() from `run()` waits until the clip has run out, as on the
    /// handset; play() from anywhere else still returns at once, so an effect cannot freeze a game.
    #[test]
    fn play_on_a_sound_thread_blocks_until_the_clip_runs_out() {
        let clock = TestClock::stepping(1);
        let result = run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), Box::new([SoundThread::as_proto()])]),
            Box::new(TestPlatform::with_clock(clock.clone())),
            move |jvm, system| async move {
                let (clip, thread) = open_sound_thread_clip(&jvm).await?;
                let length = system.audio().duration(0).unwrap();
                assert!(length >= 1000, "{length}");

                for via_helper in [false, true] {
                    jvm.put_static_field("test/SoundThread", "viaHelper", "Z", via_helper).await?;
                    let before = clock.peek();
                    let _: () = jvm.invoke_virtual(&thread, "test/SoundThread", "run", "()V", ()).await?;
                    let waited = clock.peek() - before;
                    assert!(
                        waited >= length,
                        "run()'s play() (helper {via_helper}) returned after {waited} ms of a {length} ms clip"
                    );
                    let outcome: i32 = jvm.get_static_field("test/SoundThread", "outcome", "I").await?;
                    assert_eq!(outcome, 1);
                }

                let before = clock.peek();
                let _: () = jvm.invoke_virtual(&thread, "test/SoundThread", "effect", "()V", ()).await?;
                let waited = clock.peek() - before;
                assert!(waited < length / 2, "an effect's play() waited {waited} ms");
                let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", "stop", "()V", ()).await?;

                Ok(())
            },
        );
        assert!(result.is_ok(), "{result:?}");
    }

    /// The way out of 71d1d8235bd1's deadlock: another thread's close() (its stop()) — or stop()
    /// (47fe675bfffd) — ends the sound thread's play() with `UserStopException`, and stops the sound.
    #[test]
    fn stop_or_close_ends_a_sound_threads_play_with_user_stop_exception() {
        for method in ["close", "stop"] {
            let platform = TestPlatform::with_clock(TestClock::stepping(1));
            let log = platform.audio_log();
            let result = run_jvm_test_with_system(
                Box::new([wie_midp::get_protos().into(), get_protos().into(), Box::new([SoundThread::as_proto()])]),
                Box::new(platform),
                move |jvm, _system| async move {
                    let (clip, runnable) = open_sound_thread_clip(&jvm).await?;
                    let thread = jvm.new_class("java/lang/Thread", "(Ljava/lang/Runnable;)V", (runnable,)).await?;
                    let _: () = jvm.invoke_virtual(&thread, "java/lang/Thread", "start", "()V", ()).await?;
                    let _: () = jvm.invoke_static("java/lang/Thread", "sleep", "(J)V", (100i64,)).await?;
                    let outcome: i32 = jvm.get_static_field("test/SoundThread", "outcome", "I").await?;
                    assert_eq!(outcome, 0, "play() did not wait for the clip");

                    let _: () = jvm.invoke_virtual(&clip, "net/wie/WieAudioClip", method, "()V", ()).await?;
                    let _: () = jvm.invoke_static("java/lang/Thread", "sleep", "(J)V", (100i64,)).await?;
                    let outcome: i32 = jvm.get_static_field("test/SoundThread", "outcome", "I").await?;
                    assert_eq!(outcome, 2, "{method}() did not end play() with UserStopException");

                    Ok(())
                },
            );
            assert!(result.is_ok(), "{method}: {result:?}");
            let shape: Vec<&str> = log
                .lock()
                .iter()
                .map(|command| match command {
                    AudioCommand::Play { .. } => "play",
                    AudioCommand::Stop { .. } => "stop",
                })
                .collect();
            assert_eq!(shape, ["play", "stop"], "{method}");
        }
    }
}
