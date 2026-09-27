use alloc::{vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
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
// loaded to one handle per clip.
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
        jvm.put_field(this, "repeat", "Z", repeat).await?;
        jvm.put_field(this, "paused", "Z", false).await?;
        if let Some(handle) = Self::handle(jvm, this).await?
            && let Err(error) = context.system().audio().play(handle, repeat)
        {
            tracing::error!("net.wie.WieAudioClip: failed to play audio: {error:?}");
        }

        Ok(())
    }

    async fn release(jvm: &Jvm, context: &mut WieJvmContext, this: &mut ClassInstanceRef<Self>) -> JvmResult<()> {
        if let Some(handle) = Self::loaded(jvm, this).await? {
            let _ = context.system().audio().close(handle);
            jvm.put_field(this, "audioHandle", "I", 0).await?;
        }
        jvm.put_field(this, "paused", "Z", false).await?;

        Ok(())
    }

    async fn play(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::play({this:?})");

        Self::start(jvm, context, &mut this, false).await
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

        if let Some(handle) = Self::handle(jvm, &this).await? {
            context.system().audio().stop(handle);
        }
        jvm.put_field(&mut this, "paused", "Z", false).await?;

        Ok(())
    }

    async fn close(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::close({this:?})");

        jvm.put_field(&mut this, "closed", "Z", true).await?;
        jvm.put_field(&mut this, "paused", "Z", false).await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec::Vec};

    use jvm::{Array, ClassInstanceRef, JavaError, Result as JvmResult, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::{TestPlatform, run_jvm_test, run_jvm_test_with_system};
    use wie_backend::AudioCommand;

    use crate::{classes::com::skt::m::AudioClip, get_protos};

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
}
