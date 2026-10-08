use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::{io::InputStream, lang::String};

use wie_backend::tone;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::media::Player;

// class javax.microedition.media.Manager
pub struct Manager;

impl Manager {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/media/Manager",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "createPlayer",
                    "(Ljava/io/InputStream;Ljava/lang/String;)Ljavax/microedition/media/Player;",
                    Self::create_player,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createPlayer",
                    "(Ljava/lang/String;)Ljavax/microedition/media/Player;",
                    Self::create_player_locator,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "playTone",
                    "(III)V",
                    Self::play_tone,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            // the audio handle playTone reuses, plus one (0 = none yet): a tone a key press is not a new clip each time
            fields: vec![JavaFieldProto::new(
                "toneHandle",
                "I",
                FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
            )],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL,
        }
    }

    async fn create_player(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        stream: ClassInstanceRef<InputStream>,
        r#type: ClassInstanceRef<String>,
    ) -> Result<ClassInstanceRef<Player>> {
        tracing::debug!("javax.microedition.media.Manager::createPlayer({stream:?}, {type:?})");

        if stream.is_null() {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "stream is null").await);
        }
        // A null type means «work it out from the data», which the player does (MIDI · WAV · else SMAF).
        let supported = r#type.is_null()
            || matches!(
                JavaLangString::to_rust_string(jvm, &r#type).await?.to_ascii_lowercase().as_str(),
                "application/vnd.smaf" | "audio/midi" | "audio/mid" | "audio/x-midi" | "audio/sp-midi" | "audio/x-wav" | "audio/wav"
            );
        if supported {
            Ok(jvm.new_class("net/wie/SmafPlayer", "(Ljava/io/InputStream;)V", (stream,)).await?.into())
        } else {
            Err(jvm.exception("javax/microedition/media/MediaException", "Unsupported media type").await)
        }
    }

    async fn play_tone(jvm: &Jvm, context: &mut WieJvmContext, note: i32, duration: i32, volume: i32) -> Result<()> {
        tracing::debug!("javax.microedition.media.Manager::playTone({note}, {duration}, {volume})");

        if !(0..=127).contains(&note) || duration <= 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid note or duration").await);
        }
        let sequence = tone(note, duration, volume);
        let stored: i32 = jvm.get_static_field("javax/microedition/media/Manager", "toneHandle", "I").await?;
        let mut audio = context.system().audio();
        let handle = if stored > 0 && audio.replace((stored - 1) as u32, sequence.clone()).is_ok() {
            (stored - 1) as u32
        } else {
            let handle = audio.load_sequence(sequence);
            drop(audio);
            jvm.put_static_field("javax/microedition/media/Manager", "toneHandle", "I", handle as i32 + 1)
                .await?;
            audio = context.system().audio();
            handle
        };
        let _ = audio.play(handle, false);

        Ok(())
    }

    // ponytail: the tone device plays nothing — an empty SMAF player gives it the real state
    // machine and no ToneControl. Synthesize tone sequences when a title needs the sound.
    async fn create_player_locator(jvm: &Jvm, _context: &mut WieJvmContext, locator: ClassInstanceRef<String>) -> Result<ClassInstanceRef<Player>> {
        tracing::debug!("javax.microedition.media.Manager::createPlayer({locator:?})");

        if locator.is_null() {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "locator is null").await);
        }
        if JavaLangString::to_rust_string(jvm, &locator).await? != "device://tone" {
            return Err(jvm.exception("javax/microedition/media/MediaException", "Unsupported locator").await);
        }
        let data = jvm.instantiate_array("B", 0).await?;
        let stream = jvm.new_class("java/io/ByteArrayInputStream", "([B)V", (data,)).await?;
        Ok(jvm.new_class("net/wie/SmafPlayer", "(Ljava/io/InputStream;)V", (stream,)).await?.into())
    }
}
