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

    async fn pause(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::pause({clip:?})");

        Ok(false)
    }

    async fn stop(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::stop({clip:?})");

        Ok(false)
    }

    async fn resume(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::resume({clip:?})");

        Ok(false)
    }

    async fn play(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>, repeat: bool) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::play({clip:?}, {repeat})");

        Ok(false)
    }

    async fn record(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::record({clip:?})");

        Ok(false)
    }

    async fn play_clip(jvm: &Jvm, _context: &mut WieJvmContext, clip: ClassInstanceRef<Clip>, repeat: bool) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Player::play({clip:?}, {repeat})");

        let player = Clip::player(jvm, &clip).await?;

        if !player.is_null() {
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
    async fn resume_clip(jvm: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<Clip>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Player::resume({clip:?})");

        let player = Clip::player(jvm, &clip).await?;

        if !player.is_null() {
            let started: bool = jvm.get_field(&player, "started", "Z").await?;
            if !started {
                let _: () = jvm.invoke_virtual(&player, "javax/microedition/media/Player", "start", "()V", ()).await?;
            }

            return Ok(true);
        }

        Ok(false)
    }
}

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

    // `play`/`stop`/`resume` on a NULL clip report failure; they panicked the host before.
    #[test]
    fn test_null_clip_reports_failure() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let null = || ClassInstanceRef::<Clip>::new(None);

            let stopped: bool = jvm.invoke_static("org/kwis/msp/media/Player", "stop", "(Lorg/kwis/msp/media/Clip;)Z", (null(),)).await?;
            let resumed: bool = jvm.invoke_static("org/kwis/msp/media/Player", "resume", "(Lorg/kwis/msp/media/Clip;)Z", (null(),)).await?;
            let played: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "play", "(Lorg/kwis/msp/media/Clip;Z)Z", (null(), false))
                .await?;

            assert!(!stopped && !resumed && !played);

            Ok(())
        })
    }
}
