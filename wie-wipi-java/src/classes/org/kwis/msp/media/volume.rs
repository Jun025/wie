use alloc::vec;

use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.media.Volume
pub struct Volume;

impl Volume {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/media/Volume",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "set",
                    "(I)V",
                    Self::set,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::NATIVE | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "get",
                    "()I",
                    Self::get,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::NATIVE | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("setMute", "(IZ)V", Self::set_mute, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new("getMute", "(I)Z", Self::get_mute, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "setDefaultVolume",
                    "(II)Z",
                    Self::set_default_volume,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getDefaultVolume",
                    "(I)I",
                    Self::get_default_volume,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    // The handset volume, 0..100 — what the game sets from its own sound setting (measured
    // 2026-09-27 over the local corpus: 28 KTF titles call `set`, all with 20..80). It scales all
    // of the game's sound, on top of each `Clip`'s own volume.
    async fn get(_: &Jvm, context: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.media.Volume::get()");

        Ok((context.system().audio().master_volume() * 100.0).round() as i32)
    }

    async fn set(_: &Jvm, context: &mut WieJvmContext, level: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.Volume::set({level})");

        context.system().audio().set_master_volume(level as f32 / 100.0);

        Ok(())
    }

    async fn set_mute(_: &Jvm, _: &mut WieJvmContext, volume_type: i32, mute: bool) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Volume::setMute({volume_type}, {mute})");

        Ok(())
    }

    async fn get_mute(_: &Jvm, _: &mut WieJvmContext, volume_type: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Volume::getMute({volume_type})");

        Ok(false)
    }

    async fn set_default_volume(_: &Jvm, _: &mut WieJvmContext, volume_type: i32, volume: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Volume::setDefaultVolume({volume_type}, {volume})");

        Ok(false)
    }

    async fn get_default_volume(_: &Jvm, _: &mut WieJvmContext, volume_type: i32) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.media.Volume::getDefaultVolume({volume_type})");

        Ok(0)
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    #[test]
    fn test_volume_type_stubs_return_neutral_values() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let _: () = jvm.invoke_static("org/kwis/msp/media/Volume", "setMute", "(IZ)V", (7, true)).await?;
            let muted: bool = jvm.invoke_static("org/kwis/msp/media/Volume", "getMute", "(I)Z", (7,)).await?;
            let set_default: bool = jvm
                .invoke_static("org/kwis/msp/media/Volume", "setDefaultVolume", "(II)Z", (7, 11))
                .await?;
            let default_volume: i32 = jvm.invoke_static("org/kwis/msp/media/Volume", "getDefaultVolume", "(I)I", (7,)).await?;

            assert!(!muted);
            assert!(!set_default);
            assert_eq!(default_volume, 0);

            Ok(())
        })
    }
}
