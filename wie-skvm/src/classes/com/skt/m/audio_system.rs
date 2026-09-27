use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::com::skt::m::audio_clip::AudioClip;

const MAX_VOLUME: i32 = 5;

// class com.skt.m.AudioSystem
pub struct AudioSystem;

impl AudioSystem {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/skt/m/AudioSystem",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "getAudioClip",
                    "(Ljava/lang/String;)Lcom/skt/m/AudioClip;",
                    Self::get_audio_clip,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getClipFormats",
                    "()[Ljava/lang/String;",
                    Self::get_clip_formats,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getMaxVolume",
                    "(Ljava/lang/String;)I",
                    Self::get_max_volume,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getVolume",
                    "(Ljava/lang/String;)I",
                    Self::get_volume,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "setVolume",
                    "(Ljava/lang/String;I)V",
                    Self::set_volume,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL,
        }
    }

    async fn get_audio_clip(jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<AudioClip>> {
        tracing::debug!("com.skt.m.AudioSystem::getAudioClip({name:?})");

        if name.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "format is null").await);
        }

        let audio_clip = jvm.new_class("net/wie/WieAudioClip", "(Ljava/lang/String;)V", (name,)).await?;

        Ok(audio_clip.into())
    }

    async fn get_clip_formats(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Array<String>>> {
        tracing::warn!("stub com.skt.m.AudioSystem::getClipFormats()");

        Ok(jvm.instantiate_array("Ljava/lang/String;", 0).await?.into())
    }

    // Volume is 0..MAX_VOLUME per format, and it scales all of the game's sound: every clip an SKT
    // title plays is "mmf" (see `WieAudioClip`), so one level serves every format. MAX_VOLUME is
    // KEmulator's `getMaxVolume` (its `com/skt/m/AudioSystem.class` returns `iconst_5`). It was 0
    // while stubbed, which told a game that scales its setting by the maximum to ask for silence.
    // A level above the maximum (노리타이쿤 sets 50, 사고뭉치트윈스 15) is full volume.
    async fn get_max_volume(jvm: &Jvm, _context: &mut WieJvmContext, format: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::debug!("com.skt.m.AudioSystem::getMaxVolume({format:?})");

        if format.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "format is null").await);
        }

        Ok(MAX_VOLUME)
    }

    async fn get_volume(jvm: &Jvm, context: &mut WieJvmContext, format: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::debug!("com.skt.m.AudioSystem::getVolume({format:?})");

        if format.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "format is null").await);
        }

        Ok((context.system().audio().master_volume() * MAX_VOLUME as f32).round() as i32)
    }

    async fn set_volume(jvm: &Jvm, context: &mut WieJvmContext, format: ClassInstanceRef<String>, level: i32) -> JvmResult<()> {
        tracing::debug!("com.skt.m.AudioSystem::setVolume({format:?}, {level})");

        if format.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "format is null").await);
        }

        context.system().audio().set_master_volume(level as f32 / MAX_VOLUME as f32);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, JavaError, Result as JvmResult, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::{TestPlatform, run_jvm_test, run_jvm_test_with_system};

    use crate::{classes::com::skt::m::AudioClip, get_protos};

    #[test]
    fn audio_system_rejects_null_format_arguments() {
        let result = run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let null_format = ClassInstanceRef::<String>::new(None);

            let clip_result: JvmResult<ClassInstanceRef<AudioClip>> = jvm
                .invoke_static(
                    "com/skt/m/AudioSystem",
                    "getAudioClip",
                    "(Ljava/lang/String;)Lcom/skt/m/AudioClip;",
                    (null_format.clone(),),
                )
                .await;
            let Err(JavaError::JavaException(exception)) = clip_result else {
                panic!("AudioSystem.getAudioClip accepted a null format");
            };
            assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));

            for method in ["getMaxVolume", "getVolume"] {
                let volume_result: JvmResult<i32> = jvm
                    .invoke_static("com/skt/m/AudioSystem", method, "(Ljava/lang/String;)I", (null_format.clone(),))
                    .await;
                let Err(JavaError::JavaException(exception)) = volume_result else {
                    panic!("AudioSystem.{method} accepted a null format");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));
            }

            let set_result: JvmResult<()> = jvm
                .invoke_static("com/skt/m/AudioSystem", "setVolume", "(Ljava/lang/String;I)V", (null_format, 0))
                .await;
            let Err(JavaError::JavaException(exception)) = set_result else {
                panic!("AudioSystem.setVolume accepted a null format");
            };
            assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));

            Ok(())
        });

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }

    // 0..5, and the level scales every clip. With the maximum at 0 (the stub), 21 of the 30 SKT
    // titles that set a volume read it and set 0; with 5, 20 of those 21 set a nonzero level.
    #[test]
    fn audio_system_volume_is_zero_to_five_and_sets_the_master() {
        let result = run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), get_protos().into()]),
            Box::new(TestPlatform::new()),
            |jvm, system| async move {
                let mmf: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "mmf").await?.into();
                let max: i32 = jvm
                    .invoke_static("com/skt/m/AudioSystem", "getMaxVolume", "(Ljava/lang/String;)I", (mmf.clone(),))
                    .await?;
                let initial: i32 = jvm
                    .invoke_static("com/skt/m/AudioSystem", "getVolume", "(Ljava/lang/String;)I", (mmf.clone(),))
                    .await?;
                let _: () = jvm
                    .invoke_static("com/skt/m/AudioSystem", "setVolume", "(Ljava/lang/String;I)V", (mmf.clone(), 3))
                    .await?;
                let set: i32 = jvm
                    .invoke_static("com/skt/m/AudioSystem", "getVolume", "(Ljava/lang/String;)I", (mmf,))
                    .await?;

                assert_eq!((max, initial, set), (5, 5, 3));
                assert_eq!(system.audio().master_volume(), 0.6);

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }
}
