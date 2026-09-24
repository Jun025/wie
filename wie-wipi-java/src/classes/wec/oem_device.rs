use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::wec::SYSTheme;

// class wec.OEMDevice -- a handset-vendor (OEM) extension, not part of WIPI: it is in neither
// `AromaWIPI_classes.zip` nor any SDK under `game_lab/vendor_sdk/`. Its only source is the
// reference pool of the titles that call it (월드장기체스 LGT · 귀신사냥2007 KTF), so this carries
// exactly the one member they reference and nothing else. Ownership and the full measurement:
// docs/report/0214 (why it is the handset's, not the game's) and the round that built this one.
pub struct OEMDevice;

impl OEMDevice {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "wec/OEMDevice",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            // Static: 월드장기체스 imports it as a direct (non-virtual) target with no <init> and no
            // instance anywhere to call it on -- the link record is `direct 14+3`, i.e. the two
            // class getters plus this one method.
            methods: vec![JavaMethodProto::new(
                "getSYSTheme",
                "()Lwec/SYSTheme;",
                Self::get_sys_theme,
                MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            )],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    // Never null: the one caller null-checks the result and takes the NullPointerException path
    // on null (`cmp r5, #0` → `beq` at 0x43774 in 월드장기체스's binary.mod).
    async fn get_sys_theme(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<SYSTheme>> {
        tracing::warn!("stub wec.OEMDevice::getSYSTheme()");

        Ok(jvm.new_class("wec/SYSTheme", "()V", ()).await?.into())
    }
}
