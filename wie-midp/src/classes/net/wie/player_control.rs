use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_backend::parse_tone_sequence;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::net::wie::SmafPlayer;

// class net.wie.PlayerControl — what `Player.getControl` hands out for "VolumeControl" and
// "ToneControl": volume goes to the player's audio handle, a tone sequence replaces what it plays.
pub struct PlayerControl;

impl PlayerControl {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/PlayerControl",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![
                "javax/microedition/media/control/VolumeControl",
                "javax/microedition/media/control/ToneControl",
            ],
            methods: vec![
                JavaMethodProto::new("<init>", "(Lnet/wie/SmafPlayer;)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setMute", "(Z)V", Self::set_mute, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("isMuted", "()Z", Self::is_muted, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setLevel", "(I)I", Self::set_level, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getLevel", "()I", Self::get_level, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setSequence", "([B)V", Self::set_sequence, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("player", "Lnet/wie/SmafPlayer;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("level", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("muted", "Z", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, player: ClassInstanceRef<SmafPlayer>) -> Result<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "player", "Lnet/wie/SmafPlayer;", player).await?;
        jvm.put_field(&mut this, "level", "I", 100).await
    }

    async fn handle(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> Result<u32> {
        let player: ClassInstanceRef<SmafPlayer> = jvm.get_field(this, "player", "Lnet/wie/SmafPlayer;").await?;
        let handle: i32 = jvm.get_field(&player, "audioHandle", "I").await?;

        Ok(handle as u32)
    }

    async fn apply(jvm: &Jvm, context: &mut WieJvmContext, this: &ClassInstanceRef<Self>) -> Result<()> {
        let level: i32 = jvm.get_field(this, "level", "I").await?;
        let muted: bool = jvm.get_field(this, "muted", "Z").await?;
        let volume = if muted { 0.0 } else { level as f32 / 100.0 };
        // a closed player has no handle left; its volume no longer matters
        let _ = context.system().audio().set_volume(Self::handle(jvm, this).await?, volume);

        Ok(())
    }

    async fn set_mute(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, muted: bool) -> Result<()> {
        jvm.put_field(&mut this, "muted", "Z", muted).await?;
        Self::apply(jvm, context, &this).await
    }

    async fn is_muted(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<bool> {
        jvm.get_field(&this, "muted", "Z").await
    }

    async fn set_level(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, level: i32) -> Result<i32> {
        let level = level.clamp(0, 100);
        jvm.put_field(&mut this, "level", "I", level).await?;
        Self::apply(jvm, context, &this).await?;

        Ok(level)
    }

    async fn get_level(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<i32> {
        jvm.get_field(&this, "level", "I").await
    }

    async fn set_sequence(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, sequence: ClassInstanceRef<Array<i8>>) -> Result<()> {
        if sequence.is_null() {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "sequence is null").await);
        }
        let bytes: alloc::vec::Vec<i8> = jvm.load_array(&sequence, 0, jvm.array_length(&sequence).await?).await?;
        let Some(parsed) = parse_tone_sequence(&bytes) else {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid tone sequence").await);
        };
        let length_ms = parsed.duration;
        if context.system().audio().replace(Self::handle(jvm, &this).await?, parsed).is_err() {
            return Err(jvm.exception("java/lang/IllegalStateException", "player is closed").await);
        }
        let mut player: ClassInstanceRef<SmafPlayer> = jvm.get_field(&this, "player", "Lnet/wie/SmafPlayer;").await?;
        jvm.put_field(&mut player, "lengthMs", "J", length_ms as i64).await
    }
}
