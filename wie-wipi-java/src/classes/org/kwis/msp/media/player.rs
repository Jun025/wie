use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::media::{BaseClip, Clip};

// class org.kwis.msp.media.Player
pub struct Player;

impl Player {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/media/Player",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "pause",
                    "(Lorg/kwis/msp/media/BaseClip;)Z",
                    Self::pause,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "stop",
                    "(Lorg/kwis/msp/media/BaseClip;)Z",
                    Self::stop,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "resume",
                    "(Lorg/kwis/msp/media/BaseClip;)Z",
                    Self::resume,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "play",
                    "(Lorg/kwis/msp/media/BaseClip;Z)Z",
                    Self::play,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "record",
                    "(Lorg/kwis/msp/media/BaseClip;)Z",
                    Self::record,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "play",
                    "(Lorg/kwis/msp/media/Clip;Z)Z",
                    Self::play_clip,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "stop",
                    "(Lorg/kwis/msp/media/Clip;)Z",
                    Self::stop_clip,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "resume",
                    "(Lorg/kwis/msp/media/Clip;)Z",
                    Self::resume_clip,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    // A `Clip` passed as its `BaseClip` supertype takes the Clip path: some titles are compiled
    // against the BaseClip signatures and play every sound through them (2026-09-27 silent-104
    // census: 232122cdfb92, 3412d851f78c). A bare BaseClip still reports failure. MIDP `stop`
    // keeps the media time, so it is WIPI's pause too.
    fn as_clip(jvm: &Jvm, clip: &ClassInstanceRef<BaseClip>) -> Option<ClassInstanceRef<Clip>> {
        (!clip.is_null() && jvm.is_instance(&***clip, "org/kwis/msp/media/Clip")).then(|| ClassInstanceRef::new(clip.instance.clone()))
    }

    async fn pause(jvm: &Jvm, context: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Player::pause({clip:?})");

        match Self::as_clip(jvm, &clip) {
            Some(clip) => Self::stop_clip(jvm, context, clip).await,
            None => Ok(false),
        }
    }

    async fn stop(jvm: &Jvm, context: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Player::stop({clip:?})");

        match Self::as_clip(jvm, &clip) {
            Some(clip) => Self::stop_clip(jvm, context, clip).await,
            None => Ok(false),
        }
    }

    async fn resume(jvm: &Jvm, context: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Player::resume({clip:?})");

        match Self::as_clip(jvm, &clip) {
            Some(clip) => Self::resume_clip(jvm, context, clip).await,
            None => Ok(false),
        }
    }

    async fn play(jvm: &Jvm, context: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>, repeat: bool) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Player::play({clip:?}, {repeat})");

        match Self::as_clip(jvm, &clip) {
            Some(clip) => Self::play_clip(jvm, context, clip, repeat).await,
            None => Ok(false),
        }
    }

    async fn record(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::record({clip:?})");

        Ok(false)
    }

    async fn play_clip(jvm: &Jvm, context: &mut WieJvmContext, clip: ClassInstanceRef<Clip>, repeat: bool) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Player::play({clip:?}, {repeat})");

        BaseClip::refresh(jvm, &mut ClassInstanceRef::new(clip.instance.clone())).await?;
        let player = Clip::player(jvm, &clip).await?;

        if !player.is_null() {
            Clip::apply_volume(jvm, context, &clip).await?;
            let _: () = jvm.invoke_virtual(&player, "net/wie/SmafPlayer", "start", "(Z)V", (repeat,)).await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn stop_clip(jvm: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<Clip>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Player::stop({clip:?})");

        let player = Clip::player(jvm, &clip).await?;

        if !player.is_null() {
            let _: () = jvm.invoke_virtual(&player, "javax/microedition/media/Player", "stop", "()V", ()).await?;

            return Ok(true);
        }

        Ok(false)
    }

    // MIDP has no pause/resume pair: `Player.stop()` halts playback while retaining the media
    // time, and `Player.start()` resumes from it, so WIPI's resume maps onto `start()`. The
    // target is the `javax/microedition/media/Player` interface rather than `net/wie/SmafPlayer`
    // for the same reason `stop_clip` uses it — the interface declares `start()V`. Only the
    // repeat-taking `start(Z)V` that `play_clip` needs is absent there and concrete to SmafPlayer.
    //
    // Resuming a clip that is still playing does nothing: 배틀몬스터 calls `resume` right after
    // every `play`, and restarting there cut each sound back to its first note (and, before
    // `SmafPlayer::start` kept the loop mode, turned its looping music into a one-shot).
    // "Still playing" is `getState() == STARTED`, which lets a one-shot go back to PREFETCHED
    // once its length has elapsed — so `resume` after a sound ended plays it again instead of
    // staying silent, as it did while this read the bare `started` flag.
    async fn resume_clip(jvm: &Jvm, context: &mut WieJvmContext, clip: ClassInstanceRef<Clip>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Player::resume({clip:?})");

        let player = Clip::player(jvm, &clip).await?;

        if !player.is_null() {
            Clip::apply_volume(jvm, context, &clip).await?;
            let state: i32 = jvm
                .invoke_virtual(&player, "javax/microedition/media/Player", "getState", "()I", ())
                .await?;
            if state != STARTED {
                let _: () = jvm.invoke_virtual(&player, "javax/microedition/media/Player", "start", "()V", ()).await?;
            }

            return Ok(true);
        }

        Ok(false)
    }
}

// javax.microedition.media.Player.STARTED
const STARTED: i32 = 400;

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec::Vec};

    use jvm::{ClassInstanceRef, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::{TestPlatform, run_jvm_test, run_jvm_test_with_system};
    use wie_backend::AudioCommand;
    use wie_util::Result;

    use crate::{
        classes::org::kwis::msp::media::{BaseClip, Clip},
        get_protos,
    };

    #[test]
    fn test_base_clip_overloads_return_false() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let clip: ClassInstanceRef<BaseClip> = jvm.new_class("org/kwis/msp/media/BaseClip", "()V", ()).await?.into();

            let paused: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "pause", "(Lorg/kwis/msp/media/BaseClip;)Z", (clip.clone(),))
                .await?;
            let stopped: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "stop", "(Lorg/kwis/msp/media/BaseClip;)Z", (clip.clone(),))
                .await?;
            let resumed: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "resume", "(Lorg/kwis/msp/media/BaseClip;)Z", (clip.clone(),))
                .await?;
            let played: bool = jvm
                .invoke_static(
                    "org/kwis/msp/media/Player",
                    "play",
                    "(Lorg/kwis/msp/media/BaseClip;Z)Z",
                    (clip.clone(), true),
                )
                .await?;
            let recorded: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "record", "(Lorg/kwis/msp/media/BaseClip;)Z", (clip,))
                .await?;

            assert!(!paused);
            assert!(!stopped);
            assert!(!resumed);
            assert!(!played);
            assert!(!recorded);

            Ok(())
        })
    }

    // `setBuffer` registers an array the game keeps writing into: a play after the game rewrote
    // it must load the new bytes (a new handle), a play of unchanged bytes must not.
    #[test]
    fn test_play_reloads_a_rewritten_set_buffer_array() -> Result<()> {
        let platform = TestPlatform::new();
        let log = platform.audio_log();

        run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), get_protos().into()]),
            Box::new(platform),
            |jvm, _system| async move {
                let r#type: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "audio/test").await?.into();
                let mut data = jvm.instantiate_array("B", 4).await?;
                let clip: ClassInstanceRef<Clip> = jvm
                    .new_class("org/kwis/msp/media/Clip", "(Ljava/lang/String;[B)V", (r#type, data.clone()))
                    .await?
                    .into();

                for rewrite in [false, true] {
                    if rewrite {
                        jvm.store_array(&mut data, 0, [1i8, 2, 3, 4]).await?;
                    }
                    for _ in 0..2 {
                        let _: bool = jvm
                            .invoke_static(
                                "org/kwis/msp/media/Player",
                                "play",
                                "(Lorg/kwis/msp/media/Clip;Z)Z",
                                (clip.clone(), false),
                            )
                            .await?;
                    }
                }

                Ok(())
            },
        )?;

        let handles: Vec<u32> = log
            .lock()
            .iter()
            .filter_map(|command| match command {
                AudioCommand::Play { handle, .. } => Some(*handle),
                AudioCommand::Stop { .. } => None,
            })
            .collect();
        assert_eq!(handles.len(), 4);
        assert_eq!(handles[0], handles[1]);
        assert_ne!(handles[1], handles[2]);
        assert_eq!(handles[2], handles[3]);

        Ok(())
    }

    // `putData` replaces the array the constructor registered: the game reusing that array
    // afterwards must not make a play reload it over the sound `putData` loaded (gate② #359).
    #[test]
    fn test_put_data_releases_the_constructor_array() -> Result<()> {
        let platform = TestPlatform::new();
        let log = platform.audio_log();

        run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), get_protos().into()]),
            Box::new(platform),
            |jvm, _system| async move {
                let r#type: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "audio/test").await?.into();
                let mut registered = jvm.instantiate_array("B", 4).await?;
                let clip: ClassInstanceRef<Clip> = jvm
                    .new_class("org/kwis/msp/media/Clip", "(Ljava/lang/String;[B)V", (r#type, registered.clone()))
                    .await?
                    .into();
                let mut put = jvm.instantiate_array("B", 4).await?;
                jvm.store_array(&mut put, 0, [5i8, 6, 7, 8]).await?;
                let _: i32 = jvm
                    .invoke_virtual(&clip, "org/kwis/msp/media/BaseClip", "putData", "([BII)I", (put, 0, 4))
                    .await?;

                for _ in 0..2 {
                    let _: bool = jvm
                        .invoke_static(
                            "org/kwis/msp/media/Player",
                            "play",
                            "(Lorg/kwis/msp/media/Clip;Z)Z",
                            (clip.clone(), false),
                        )
                        .await?;
                    jvm.store_array(&mut registered, 0, [1i8, 2, 3, 4]).await?;
                }

                Ok(())
            },
        )?;

        let handles: Vec<u32> = log
            .lock()
            .iter()
            .filter_map(|command| match command {
                AudioCommand::Play { handle, .. } => Some(*handle),
                AudioCommand::Stop { .. } => None,
            })
            .collect();
        assert_eq!(handles.len(), 2);
        assert_eq!(handles[0], handles[1]);

        Ok(())
    }

    // A Clip handed over as its BaseClip supertype sounds: the BaseClip overloads were stubs,
    // and the titles compiled against them were silent.
    #[test]
    fn test_clip_through_base_clip_overloads_plays() -> Result<()> {
        let platform = TestPlatform::new();
        let log = platform.audio_log();

        run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), get_protos().into()]),
            Box::new(platform),
            |jvm, _system| async move {
                let r#type: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "audio/test").await?.into();
                let data = jvm.instantiate_array("B", 0).await?;
                let clip: ClassInstanceRef<BaseClip> = jvm
                    .new_class("org/kwis/msp/media/Clip", "(Ljava/lang/String;[B)V", (r#type, data))
                    .await?
                    .into();

                let played: bool = jvm
                    .invoke_static(
                        "org/kwis/msp/media/Player",
                        "play",
                        "(Lorg/kwis/msp/media/BaseClip;Z)Z",
                        (clip.clone(), true),
                    )
                    .await?;
                let stopped: bool = jvm
                    .invoke_static("org/kwis/msp/media/Player", "stop", "(Lorg/kwis/msp/media/BaseClip;)Z", (clip,))
                    .await?;
                assert!(played && stopped);

                Ok(())
            },
        )?;

        let shape: Vec<(&str, bool)> = log
            .lock()
            .iter()
            .map(|command| match command {
                AudioCommand::Play { repeat, .. } => ("play", *repeat),
                AudioCommand::Stop { .. } => ("stop", false),
            })
            .collect();
        assert_eq!(shape, [("play", true), ("stop", false)]);

        Ok(())
    }

    #[test]
    fn test_clip_playback_uses_repeat_overload() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let r#type: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "audio/test").await?.into();
            let data = jvm.instantiate_array("B", 0).await?;
            let clip: ClassInstanceRef<Clip> = jvm
                .new_class("org/kwis/msp/media/Clip", "(Ljava/lang/String;[B)V", (r#type, data))
                .await?
                .into();

            let played: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "play", "(Lorg/kwis/msp/media/Clip;Z)Z", (clip.clone(), true))
                .await?;
            let stopped: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "stop", "(Lorg/kwis/msp/media/Clip;)Z", (clip,))
                .await?;

            assert!(played);
            assert!(stopped);

            Ok(())
        })
    }

    // The Clip overload is the one 배틀몬스터 calls; the BaseClip overload above it is a bare
    // stub, so asserting only that one would pass with the Clip signature still unregistered.
    #[test]
    fn test_clip_resume_delegates_to_backing_player() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let r#type: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "audio/test").await?.into();
            let data = jvm.instantiate_array("B", 0).await?;
            let clip: ClassInstanceRef<Clip> = jvm
                .new_class("org/kwis/msp/media/Clip", "(Ljava/lang/String;[B)V", (r#type, data))
                .await?
                .into();

            let stopped: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "stop", "(Lorg/kwis/msp/media/Clip;)Z", (clip.clone(),))
                .await?;
            let resumed: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "resume", "(Lorg/kwis/msp/media/Clip;)Z", (clip,))
                .await?;

            assert!(stopped);
            assert!(resumed);

            Ok(())
        })
    }

    // play → resume (배틀몬스터's pattern) must not replay, and stop → resume must keep the loop
    // mode the clip was played with. Both reached the sink wrong until 2026-09-27.
    #[test]
    fn test_clip_resume_keeps_playing_clip_and_loop_mode() -> Result<()> {
        let platform = TestPlatform::new();
        let log = platform.audio_log();

        run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), get_protos().into()]),
            Box::new(platform),
            |jvm, _system| async move {
                let r#type: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "audio/test").await?.into();
                let data = jvm.instantiate_array("B", 0).await?;
                let clip: ClassInstanceRef<Clip> = jvm
                    .new_class("org/kwis/msp/media/Clip", "(Ljava/lang/String;[B)V", (r#type, data))
                    .await?
                    .into();

                let _: bool = jvm
                    .invoke_static("org/kwis/msp/media/Player", "play", "(Lorg/kwis/msp/media/Clip;Z)Z", (clip.clone(), true))
                    .await?;
                let _: bool = jvm
                    .invoke_static("org/kwis/msp/media/Player", "resume", "(Lorg/kwis/msp/media/Clip;)Z", (clip.clone(),))
                    .await?;
                let _: bool = jvm
                    .invoke_static("org/kwis/msp/media/Player", "stop", "(Lorg/kwis/msp/media/Clip;)Z", (clip.clone(),))
                    .await?;
                let _: bool = jvm
                    .invoke_static("org/kwis/msp/media/Player", "resume", "(Lorg/kwis/msp/media/Clip;)Z", (clip,))
                    .await?;

                Ok(())
            },
        )?;

        let shape: Vec<(&str, bool)> = log
            .lock()
            .iter()
            .map(|command| match command {
                AudioCommand::Play { repeat, .. } => ("play", *repeat),
                AudioCommand::Stop { .. } => ("stop", false),
            })
            .collect();
        assert_eq!(shape, [("play", true), ("stop", false), ("play", true)]);

        Ok(())
    }

    // A one-shot that has ended is not "still playing": resume must play it again. The empty
    // clip parses to length 0, so it has ended the moment it starts. Until 2026-09-27 `resume`
    // read a `started` flag that only an explicit stop cleared, and this sent nothing.
    #[test]
    fn test_clip_resume_replays_a_finished_one_shot() -> Result<()> {
        let platform = TestPlatform::new();
        let log = platform.audio_log();

        run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), get_protos().into()]),
            Box::new(platform),
            |jvm, _system| async move {
                let r#type: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "audio/test").await?.into();
                let data = jvm.instantiate_array("B", 0).await?;
                let clip: ClassInstanceRef<Clip> = jvm
                    .new_class("org/kwis/msp/media/Clip", "(Ljava/lang/String;[B)V", (r#type, data))
                    .await?
                    .into();

                let _: bool = jvm
                    .invoke_static(
                        "org/kwis/msp/media/Player",
                        "play",
                        "(Lorg/kwis/msp/media/Clip;Z)Z",
                        (clip.clone(), false),
                    )
                    .await?;
                let _: bool = jvm
                    .invoke_static("org/kwis/msp/media/Player", "resume", "(Lorg/kwis/msp/media/Clip;)Z", (clip,))
                    .await?;

                Ok(())
            },
        )?;

        let shape: Vec<(&str, bool)> = log
            .lock()
            .iter()
            .map(|command| match command {
                AudioCommand::Play { repeat, .. } => ("play", *repeat),
                AudioCommand::Stop { .. } => ("stop", false),
            })
            .collect();
        // The replay restarts the handle, which the backend does as stop + play.
        assert_eq!(shape, [("play", false), ("stop", false), ("play", false)]);

        Ok(())
    }

    // A Clip with no backing player must report failure rather than panic on the null deref --
    // the same contract play_clip/stop_clip hold, and the branch resume_clip shares with them.
    #[test]
    fn test_clip_resume_without_player_reports_failure() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let r#type: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "audio/test").await?.into();
            let clip: ClassInstanceRef<Clip> = jvm.new_class("org/kwis/msp/media/Clip", "(Ljava/lang/String;)V", (r#type,)).await?.into();

            let resumed: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "resume", "(Lorg/kwis/msp/media/Clip;)Z", (clip,))
                .await?;

            assert!(!resumed);

            Ok(())
        })
    }

    // A volume set before the clip has data reaches the handle when it plays; `Volume` is the
    // master on top. `getVolume` starts at 100, not the field's 0.
    #[test]
    fn test_clip_and_handset_volume_reach_the_audio_handle() -> Result<()> {
        run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), get_protos().into()]),
            Box::new(TestPlatform::new()),
            |jvm, system| async move {
                let r#type: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "audio/test").await?.into();
                let clip: ClassInstanceRef<Clip> = jvm.new_class("org/kwis/msp/media/Clip", "(Ljava/lang/String;)V", (r#type,)).await?.into();

                let initial: i32 = jvm.invoke_virtual(&clip, "org/kwis/msp/media/Clip", "getVolume", "()I", ()).await?;
                let accepted: bool = jvm.invoke_virtual(&clip, "org/kwis/msp/media/Clip", "setVolume", "(I)Z", (40,)).await?;
                let data = jvm.instantiate_array("B", 0).await?;
                let _: () = jvm
                    .invoke_virtual(&clip, "org/kwis/msp/media/Clip", "setBuffer", "([BI)V", (data, 0))
                    .await?;
                let _: bool = jvm
                    .invoke_static(
                        "org/kwis/msp/media/Player",
                        "play",
                        "(Lorg/kwis/msp/media/Clip;Z)Z",
                        (clip.clone(), false),
                    )
                    .await?;
                let player = Clip::player(&jvm, &clip).await?;
                let handle: i32 = jvm.get_field(&player, "audioHandle", "I").await?;

                let _: () = jvm.invoke_static("org/kwis/msp/media/Volume", "set", "(I)V", (60,)).await?;
                let handset: i32 = jvm.invoke_static("org/kwis/msp/media/Volume", "get", "()I", ()).await?;

                assert_eq!(initial, 100);
                assert!(accepted);
                assert_eq!(system.audio().volume(handle as u32), 0.4);
                assert_eq!(handset, 60);
                assert_eq!(system.audio().master_volume(), 0.6);

                Ok(())
            },
        )
    }

    // `play`/`stop`/`resume` on a NULL clip report failure; they panicked the host before.
    #[test]
    fn test_null_clip_reports_failure() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let null = || ClassInstanceRef::<Clip>::new(None);

            let stopped: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "stop", "(Lorg/kwis/msp/media/Clip;)Z", (null(),))
                .await?;
            let resumed: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "resume", "(Lorg/kwis/msp/media/Clip;)Z", (null(),))
                .await?;
            let played: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "play", "(Lorg/kwis/msp/media/Clip;Z)Z", (null(), false))
                .await?;

            assert!(!stopped && !resumed && !played);

            Ok(())
        })
    }
}
