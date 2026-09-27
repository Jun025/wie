use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::media::Player;

// not in reference, but called by some apps..
// class org.kwis.msp.media.BaseClip
pub struct BaseClip;

impl BaseClip {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/media/BaseClip",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("putData", "([BII)I", Self::put_data, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setBuffer", "([BI)Z", Self::set_buffer, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("clearData", "()V", Self::clear_data, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("availableDataSize", "()I", Self::available_data_size, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("player", "Ljavax/microedition/media/Player;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("buffer", "[B", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("bufferSize", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("bufferHash", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.BaseClip::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn available_data_size(_jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.media.BaseClip::availableDataSize({this:?})");

        Ok(10000000 as _)
    }

    // A game's own `putData` replaces whatever `setBuffer` (or the `Clip(type, byte[])`
    // constructor, which calls it) registered: the clip stops watching that array, or a later
    // rewrite of it would make `refresh` reload the old array over the sound the game just put.
    async fn put_data(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        buffer: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.media.Clip::putData({this:?}, {buffer:?}, {offset}, {length})");

        jvm.put_field(&mut this, "buffer", "[B", None).await?;
        Self::load(jvm, &mut this, buffer, offset, length).await
    }

    async fn load(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, buffer: ClassInstanceRef<Array<i8>>, offset: i32, length: i32) -> JvmResult<i32> {
        let input_stream = jvm.new_class("java/io/ByteArrayInputStream", "([BII)V", (buffer, offset, length)).await?;
        let r#type = JavaLangString::from_rust_string(jvm, "application/vnd.smaf").await?;

        let player: ClassInstanceRef<Player> = jvm
            .invoke_static(
                "javax/microedition/media/Manager",
                "createPlayer",
                "(Ljava/io/InputStream;Ljava/lang/String;)Ljavax/microedition/media/Player;",
                (input_stream, r#type),
            )
            .await?;

        jvm.put_field(this, "player", "Ljavax/microedition/media/Player;", player).await?;

        Ok(length)
    }

    async fn clear_data(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.BaseClip::clearData({this:?})");

        let player: ClassInstanceRef<Player> = jvm.get_field(&this, "player", "Ljavax/microedition/media/Player;").await?;
        if player.is_null() {
            return Ok(());
        }

        let _: () = jvm.invoke_virtual(&player, "javax/microedition/media/Player", "close", "()V", ()).await?;

        jvm.put_field(&mut this, "player", "Ljavax/microedition/media/Player;", None).await?;
        jvm.put_field(&mut this, "buffer", "[B", None).await?;

        Ok(())
    }

    // `setBuffer` hands the clip an array the game keeps writing into: 4451036a7fb6 registers one
    // 14,636-byte array once, then copies each sound into it and calls `Player.play` — 123 plays
    // in 90 s, every one parsed from the array's first contents (a PNG) and so empty (2026-09-27
    // silent-104 census). So the clip keeps the array and `refresh` reloads it when it changed.
    async fn set_buffer(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        buffer: ClassInstanceRef<Array<i8>>,
        size: i32,
    ) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.BaseClip::setBuffer({this:?}, {buffer:?}, {size})");

        let written = Self::load(jvm, &mut this, buffer.clone(), 0, size).await?;
        jvm.put_field(&mut this, "buffer", "[B", buffer).await?;
        jvm.put_field(&mut this, "bufferSize", "I", size).await?;
        let hash = Self::buffer_hash(jvm, &this).await?;
        jvm.put_field(&mut this, "bufferHash", "I", hash).await?;

        Ok(written == size)
    }

    /// Reloads the `setBuffer` (or `Clip(type, byte[])`) array if the game rewrote it since it was last loaded.
    pub async fn refresh(jvm: &Jvm, this: &mut ClassInstanceRef<Self>) -> JvmResult<()> {
        if this.is_null() {
            return Ok(());
        }
        let buffer: ClassInstanceRef<Array<i8>> = jvm.get_field(this, "buffer", "[B").await?;
        if buffer.is_null() {
            return Ok(());
        }
        let hash = Self::buffer_hash(jvm, this).await?;
        let loaded: i32 = jvm.get_field(this, "bufferHash", "I").await?;
        if hash == loaded {
            return Ok(());
        }

        let player: ClassInstanceRef<Player> = jvm.get_field(this, "player", "Ljavax/microedition/media/Player;").await?;
        if !player.is_null() {
            let _: () = jvm.invoke_virtual(&player, "javax/microedition/media/Player", "close", "()V", ()).await?;
        }
        let size: i32 = jvm.get_field(this, "bufferSize", "I").await?;
        Self::load(jvm, this, buffer, 0, size).await?;
        jvm.put_field(this, "bufferHash", "I", hash).await
    }

    // FNV-1a over the registered bytes.
    async fn buffer_hash(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<i32> {
        let buffer: ClassInstanceRef<Array<i8>> = jvm.get_field(this, "buffer", "[B").await?;
        let size: i32 = jvm.get_field(this, "bufferSize", "I").await?;
        let length = (size.max(0) as usize).min(jvm.array_length(&buffer).await?);
        let bytes: alloc::vec::Vec<i8> = jvm.load_array(&buffer, 0, length).await?;

        Ok(bytes
            .iter()
            .fold(0x811c_9dc5u32, |hash, &byte| (hash ^ byte as u8 as u32).wrapping_mul(0x0100_0193)) as i32)
    }
}
