use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::lcdui::Display;

// Rows a shown status strip takes from the top of a 240-wide KTF screen. Measured in the corpus, not
// known from a spec: of the 240×320 KTF packages whose largest full-width image is near the screen
// height, the most common height is 296 (10 titles) against 320 (5), and d1e0badfce82 sizes a table
// for at most 319 rows. 24 is also LGT's strip at this width (`wie-lgt` graphics). docs/report/0431.
// ponytail: one height for every 240-wide handset; other widths keep 0 (176-wide images peak at 202
// = 220 − 18, measured but not paired).
const STRIP_HEIGHT_240: i32 = 24;

// class org.kwis.msp.lwc.AnnunciatorComponent
pub struct AnnunciatorComponent;

impl AnnunciatorComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/AnnunciatorComponent",
            parent_class: Some("org/kwis/msp/lwc/ShellComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Z)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("show", "()V", Self::show, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getWidth", "()I", Self::get_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getHeight", "()I", Self::get_size, MethodAccessFlags::PUBLIC),
            ],
            // Static, not an instance field: lwc instance fields shift an LGT AOT subclass's offsets.
            fields: vec![JavaFieldProto::new(
                "shownHeight",
                "I",
                FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
            )],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<AnnunciatorComponent>, a0: bool) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.AnnunciatorComponent::<init>({this:?}, {a0})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/ShellComponent", "<init>", "()V", ()).await?;

        Ok(())
    }

    // The strip is still not drawn; showing it reserves its rows, so cards made after this start below
    // it and the display reports the height left over (KTF titles call show() before new Card()).
    async fn show(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.AnnunciatorComponent::show()");

        let display: ClassInstanceRef<Display> = jvm
            .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", [])
            .await?;
        if display.is_null() {
            return Ok(());
        }
        let width: i32 = jvm.invoke_virtual(&display, "org/kwis/msp/lcdui/Display", "getWidth", "()I", ()).await?;
        let height = if width == 240 { STRIP_HEIGHT_240 } else { 0 };
        jvm.put_static_field("org/kwis/msp/lwc/AnnunciatorComponent", "shownHeight", "I", height)
            .await
    }

    /// Rows a shown status strip reserves at the top of the screen (0 until one is shown).
    pub async fn shown_height(jvm: &Jvm) -> JvmResult<i32> {
        jvm.get_static_field("org/kwis/msp/lwc/AnnunciatorComponent", "shownHeight", "I").await
    }

    // The status strip is never drawn here (show above is a stub), so it takes no room — the answer
    // Component gave before ShellComponent learned to answer the screen size. b475b6399684 reads the
    // strip's height twice at boot; answered with the screen height it stopped painting after two
    // frames (30-second probe, 2 of 2 runs), where 0 keeps it at ~240 paints.
    async fn get_size(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<AnnunciatorComponent>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.AnnunciatorComponent::getWidth/getHeight({this:?})");

        Ok(0)
    }
}
