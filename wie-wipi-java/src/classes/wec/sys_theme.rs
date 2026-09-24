use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class wec.SYSTheme -- handset OEM extension (see `OEMDevice`). A minimal stub, and it is
// allowed to be one only because of what the guest does with it, measured by disassembling
// 월드장기체스's binary.mod (LGT AOT-Java):
//
// - Its two static-field imports are the whole game's static-field imports (link record
//   `static 0+2`), and there is exactly ONE read site, 0x436a0: FORMAT_SOUND_MA3 → r7,
//   ITEM_GROUP_SOUND_DEFAULT → r6. Both registers go unmodified into the single saveItem call at
//   0x437b4 as arguments 2 and 1, and are never compared or branched on. So the game never
//   decides anything from their values; only saveItem sees them.
// - saveItem's return value is discarded: r0 is overwritten (`mov r0, r5`) straight after the call.
//
// Hence the fields are declared with the type the import table records (`I`, via the shared
// one-byte descriptor string that made them look descriptor-less to a string-pool scan) and are
// left at the JVM default. NO VALUE IS ASSIGNED, because the real ones are unknown -- there is no
// authoritative source for this class. If a caller that branches on them ever appears, this stub
// is wrong for it; the values have to come from a real handset trace, not from here.
pub struct SYSTheme;

impl SYSTheme {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "wec/SYSTheme",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                // Not referenced by any title -- the only way the guest obtains an instance is
                // OEMDevice.getSYSTheme(), which constructs one through this.
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PRIVATE),
                JavaMethodProto::new("saveItem", "(IILjava/lang/String;[BI)I", Self::save_item, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("ITEM_GROUP_SOUND_DEFAULT", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("FORMAT_SOUND_MA3", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    // "Store this sound in the handset's system theme" -- a device setting, not game state, and
    // there is no handset theme to store it in. Nothing is saved. The return value is discarded
    // by the one caller (see the header), so 0 is not a claim about what the real method returns.
    #[allow(clippy::too_many_arguments)] // the arity is the Java descriptor's
    async fn save_item(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        group: i32,
        format: i32,
        name: ClassInstanceRef<String>,
        data: ClassInstanceRef<Array<i8>>,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::warn!("stub wec.SYSTheme::saveItem({this:?}, {group}, {format}, {name:?}, {data:?}, {length})");

        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, runtime::JavaLangString};

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    use super::SYSTheme;

    // The path 월드장기체스 takes: a theme from the static getter, then saveItem with the two statics
    // as arguments. Invoking saveItem on a null theme would fail, so this also pins "never null".
    #[test]
    fn get_sys_theme_then_save_item_with_the_static_fields() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let theme: ClassInstanceRef<SYSTheme> = jvm.invoke_static("wec/OEMDevice", "getSYSTheme", "()Lwec/SYSTheme;", ()).await?;

            let group: i32 = jvm.get_static_field("wec/SYSTheme", "ITEM_GROUP_SOUND_DEFAULT", "I").await?;
            let format: i32 = jvm.get_static_field("wec/SYSTheme", "FORMAT_SOUND_MA3", "I").await?;
            let name = JavaLangString::from_rust_string(&jvm, "ring").await?;
            let data = jvm.instantiate_array("B", 1).await?;
            let _: i32 = jvm
                .invoke_virtual(
                    &theme,
                    "wec/SYSTheme",
                    "saveItem",
                    "(IILjava/lang/String;[BI)I",
                    (group, format, name, data, 1),
                )
                .await?;

            Ok(())
        })
    }
}
