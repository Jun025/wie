use alloc::vec;

use smaf_player::{SmafEvent, parse_smaf};

use jvm::{
    Array, ClassInstanceRef, Jvm, Result,
    runtime::{JavaIoInputStream, JavaLangString},
};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::{io::InputStream, lang::String};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::media::{Control, PlayerListener};

// class net.wie.SmafPlayer
pub struct SmafPlayer;

impl SmafPlayer {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/SmafPlayer",
            parent_class: Some("java/lang/Object"),
            interfaces: vec!["javax/microedition/media/Player"],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/io/InputStream;)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("realize", "()V", Self::realize, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("prefetch", "()V", Self::prefetch, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("start", "()V", Self::start, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("start", "(Z)V", Self::start_with_repeat, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("stop", "()V", Self::stop, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("deallocate", "()V", Self::deallocate, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("close", "()V", Self::close, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setMediaTime", "(J)J", Self::set_media_time, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getMediaTime", "()J", Self::get_media_time, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getState", "()I", Self::get_state, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getDuration", "()J", Self::get_duration, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getContentType",
                    "()Ljava/lang/String;",
                    Self::get_content_type,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("setLoopCount", "(I)V", Self::set_loop_count, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "addPlayerListener",
                    "(Ljavax/microedition/media/PlayerListener;)V",
                    Self::add_player_listener,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "removePlayerListener",
                    "(Ljavax/microedition/media/PlayerListener;)V",
                    Self::remove_player_listener,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "getControl",
                    "(Ljava/lang/String;)Ljavax/microedition/media/Control;",
                    Self::get_control,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "getControls",
                    "()[Ljavax/microedition/media/Control;",
                    Self::get_controls,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            // `repeat`: the loop mode `start()` resumes with — MIDP's `start()` carries none, so
            // it is whatever `start(Z)`/`setLoopCount` last set. `started`: between a start and a
            // stop. `startedAt`/`lengthMs`: when that start was and how long the clip is, so a
            // one-shot counts as finished once its length has elapsed — the sink never reports
            // the natural end. `getState` reads all four (see `playing`).
            fields: vec![
                JavaFieldProto::new("audioHandle", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("repeat", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("started", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("startedAt", "J", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("lengthMs", "J", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, stream: ClassInstanceRef<InputStream>) -> Result<()> {
        tracing::debug!("net.wie.SmafPlayer::<init>({this:?}, {stream:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        let data = JavaIoInputStream::read_until_end(jvm, &stream).await?;
        let audio_handle = context.system().audio().load_smaf(&data).unwrap();

        jvm.put_field(&mut this, "audioHandle", "I", audio_handle as i32).await?;
        jvm.put_field(&mut this, "lengthMs", "J", sequence_length_ms(&parse_smaf(&data)) as i64)
            .await?;

        Ok(())
    }

    // `start()` keeps the loop mode instead of forcing a one-shot: 배틀몬스터 starts its field
    // music with `Player.play(clip, true)` and immediately calls `Player.resume(clip)`, which maps
    // here — and until 2026-09-27 that replayed the music with `repeat = false`, so it never
    // looped (measured in the browser: `play R` → `stop` → `play` in the same instant).
    async fn start(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        let repeat: bool = jvm.get_field(&this, "repeat", "Z").await?;

        Self::start_with_repeat(jvm, context, this, repeat).await
    }

    async fn start_with_repeat(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, repeat: bool) -> Result<()> {
        tracing::debug!("net.wie.SmafPlayer::start({this:?}, {repeat})");

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;
        jvm.put_field(&mut this, "repeat", "Z", repeat).await?;
        jvm.put_field(&mut this, "started", "Z", true).await?;
        let now = context.system().platform().now().raw() as i64;
        jvm.put_field(&mut this, "startedAt", "J", now).await?;

        context.system().audio().play(audio_handle as u32, repeat).unwrap();

        Ok(())
    }

    async fn stop(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::debug!("net.wie.SmafPlayer::stop({this:?})");

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;
        jvm.put_field(&mut this, "started", "Z", false).await?;

        let system = context.system();

        system.audio().stop(audio_handle as u32);

        Ok(())
    }

    async fn close(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::debug!("net.wie.SmafPlayer::close({this:?})");

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;

        let system = context.system();

        system.audio().close(audio_handle as u32).unwrap();

        Ok(())
    }

    async fn realize(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::warn!("stub net.wie.SmafPlayer::realize({this:?})");

        Ok(())
    }

    async fn prefetch(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::warn!("stub net.wie.SmafPlayer::prefetch({this:?})");

        Ok(())
    }

    async fn deallocate(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::warn!("stub net.wie.SmafPlayer::deallocate({this:?})");

        Ok(())
    }

    async fn set_media_time(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, now: i64) -> Result<i64> {
        tracing::warn!("stub net.wie.SmafPlayer::setMediaTime({this:?}, {now})");

        Err(jvm.exception("javax/microedition/media/MediaException", "Seeking is not supported").await)
    }

    async fn get_media_time(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<i64> {
        tracing::warn!("stub net.wie.SmafPlayer::getMediaTime({this:?})");

        Ok(-1)
    }

    // STARTED while playing, PREFETCHED otherwise — MIDP returns a player to PREFETCHED when
    // its media ends, and WIPI's `Player.resume` asks this to decide whether to restart (see
    // `resume_clip`). The lifecycle states below PREFETCHED are not modelled.
    async fn get_state(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<i32> {
        tracing::debug!("net.wie.SmafPlayer::getState({this:?})");

        let started: bool = jvm.get_field(&this, "started", "Z").await?;
        let repeat: bool = jvm.get_field(&this, "repeat", "Z").await?;
        let started_at: i64 = jvm.get_field(&this, "startedAt", "J").await?;
        let length_ms: i64 = jvm.get_field(&this, "lengthMs", "J").await?;
        let now = context.system().platform().now().raw() as i64;

        Ok(if playing(started, repeat, now - started_at, length_ms) {
            STARTED
        } else {
            PREFETCHED
        })
    }

    async fn get_duration(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<i64> {
        tracing::warn!("stub net.wie.SmafPlayer::getDuration({this:?})");

        Ok(-1)
    }

    async fn get_content_type(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<ClassInstanceRef<String>> {
        tracing::debug!("net.wie.SmafPlayer::getContentType({this:?})");

        Ok(JavaLangString::from_rust_string(jvm, "application/vnd.smaf").await?.into())
    }

    // The sink has "once" and "forever" only, so any count other than 1 loops.
    async fn set_loop_count(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, count: i32) -> Result<()> {
        tracing::debug!("net.wie.SmafPlayer::setLoopCount({this:?}, {count})");

        if count == 0 || count < -1 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid loop count").await);
        }
        jvm.put_field(&mut this, "repeat", "Z", count != 1).await?;

        Ok(())
    }

    async fn add_player_listener(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<PlayerListener>,
    ) -> Result<()> {
        tracing::warn!("stub net.wie.SmafPlayer::addPlayerListener({this:?}, {listener:?})");

        Ok(())
    }

    async fn remove_player_listener(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<PlayerListener>,
    ) -> Result<()> {
        tracing::warn!("stub net.wie.SmafPlayer::removePlayerListener({this:?}, {listener:?})");

        Ok(())
    }

    async fn get_control(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        control_type: ClassInstanceRef<String>,
    ) -> Result<ClassInstanceRef<Control>> {
        tracing::warn!("stub net.wie.SmafPlayer::getControl({this:?}, {control_type:?})");

        if control_type.is_null() {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Control type is null").await);
        }

        Ok(None.into())
    }

    async fn get_controls(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<ClassInstanceRef<Array<Control>>> {
        tracing::warn!("stub net.wie.SmafPlayer::getControls({this:?})");

        Ok(jvm.instantiate_array("Ljavax/microedition/media/Control;", 0).await?.into())
    }
}

const PREFETCHED: i32 = 300;
const STARTED: i32 = 400;

/// Whether a start is still sounding `elapsed_ms` later. A loop sounds until it is stopped; a
/// one-shot until its length has elapsed. Until 2026-09-27 a one-shot counted as playing until an
/// explicit stop, so a title that restarts a finished sound with WIPI `Player.resume` got silence.
fn playing(started: bool, repeat: bool, elapsed_ms: i64, length_ms: i64) -> bool {
    started && (repeat || elapsed_ms < length_ms)
}

/// How long a parsed clip sounds, in ms — the same length the featurephone worklet loops on
/// (`audio_worklet.js` `load`): the last event, or the end of the longest PCM sample if that is
/// later.
fn sequence_length_ms(events: &[(usize, SmafEvent)]) -> u64 {
    events
        .iter()
        .map(|(time, event)| {
            let end = match event {
                SmafEvent::Wave {
                    channel,
                    sampling_rate,
                    data,
                } if *sampling_rate > 0 => (data.len() as u64 / (*channel).max(1) as u64) * 1000 / *sampling_rate as u64,
                _ => 0,
            };
            *time as u64 + end
        })
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{Array, ClassInstanceRef, JavaError, runtime::JavaLangString};
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{
        classes::javax::microedition::media::{Control, Player},
        get_protos,
    };

    #[test]
    fn test_unsupported_player_controls() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let data = jvm.instantiate_array("B", 0).await?;
            let stream = jvm.new_class("java/io/ByteArrayInputStream", "([B)V", (data,)).await?;
            let content_type = JavaLangString::from_rust_string(&jvm, "application/vnd.smaf").await?;
            let player: ClassInstanceRef<Player> = jvm
                .invoke_static(
                    "javax/microedition/media/Manager",
                    "createPlayer",
                    "(Ljava/io/InputStream;Ljava/lang/String;)Ljavax/microedition/media/Player;",
                    (stream, content_type),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(&player, "javax/microedition/media/Player", "realize", "()V", ())
                .await?;

            let controls: ClassInstanceRef<Array<Control>> = jvm
                .invoke_virtual(
                    &player,
                    "javax/microedition/media/Controllable",
                    "getControls",
                    "()[Ljavax/microedition/media/Control;",
                    (),
                )
                .await?;
            assert_eq!(jvm.array_length(&controls).await?, 0);

            let control_type = JavaLangString::from_rust_string(&jvm, "VolumeControl").await?;
            let control: ClassInstanceRef<Control> = jvm
                .invoke_virtual(
                    &player,
                    "javax/microedition/media/Player",
                    "getControl",
                    "(Ljava/lang/String;)Ljavax/microedition/media/Control;",
                    (control_type,),
                )
                .await?;
            assert!(control.is_null());

            let JavaError::JavaException(exception) = jvm
                .invoke_virtual::<_, ClassInstanceRef<Control>>(
                    &player,
                    "javax/microedition/media/Controllable",
                    "getControl",
                    "(Ljava/lang/String;)Ljavax/microedition/media/Control;",
                    (None,),
                )
                .await
                .unwrap_err();
            assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

            let _: () = jvm
                .invoke_virtual(&player, "javax/microedition/media/Player", "setLoopCount", "(I)V", (-1,))
                .await?;
            let JavaError::JavaException(exception) = jvm
                .invoke_virtual::<_, ()>(&player, "javax/microedition/media/Player", "setLoopCount", "(I)V", (0,))
                .await
                .unwrap_err();
            assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

            let JavaError::JavaException(exception) = jvm
                .invoke_virtual::<_, i64>(&player, "javax/microedition/media/Player", "setMediaTime", "(J)J", (0i64,))
                .await
                .unwrap_err();
            assert!(jvm.is_instance(&*exception, "javax/microedition/media/MediaException"));

            let _: () = jvm.invoke_virtual(&player, "javax/microedition/media/Player", "close", "()V", ()).await?;
            Ok(())
        })
    }

    #[test]
    fn test_one_shot_finishes_after_its_length_and_loops_never_do() {
        use super::playing;

        assert!(playing(true, false, 0, 500));
        assert!(playing(true, false, 499, 500));
        assert!(!playing(true, false, 500, 500));
        assert!(playing(true, true, 1_000_000, 500));
        assert!(!playing(false, true, 0, 500));
        // An unparseable clip has length 0: a one-shot of it is over at once.
        assert!(!playing(true, false, 0, 0));
    }

    #[test]
    fn test_sequence_length_covers_the_longest_pcm_tail() {
        use alloc::vec;

        use smaf_player::SmafEvent;

        use super::sequence_length_ms;

        assert_eq!(sequence_length_ms(&[]), 0);
        let events = [
            (
                0,
                SmafEvent::MidiNoteOn {
                    channel: 0,
                    note: 60,
                    velocity: 100,
                },
            ),
            (
                900,
                SmafEvent::MidiNoteOff {
                    channel: 0,
                    note: 60,
                    velocity: 0,
                },
            ),
            // 8000 stereo frames at 8 kHz = 1000 ms, starting at 100 ms: ends at 1100 ms.
            (
                100,
                SmafEvent::Wave {
                    channel: 2,
                    sampling_rate: 8000,
                    data: vec![0; 16000],
                },
            ),
            (1000, SmafEvent::End),
        ];
        assert_eq!(sequence_length_ms(&events), 1100);
    }
}
