use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.ktf.kfc.GProgressBar -- KTF's own form toolkit, like ChoiceText; no API document is in
// this repo. 83fc429f9cbe (KTF, relocated client.bin) is the only corpus title that names it, and an
// execution trace of its boot is the whole evidence for this shape:
//
// - It resolves exactly three methods, all from `startApp`: `<init>()V`, `setMaximum(I)Z` (once, 8) and
//   `setValue(I)Z` (1..8 — one per loading step). Without the class, the load failed, the title caught
//   the error and hit a NullPointerException on the null bar; with it, startApp finishes and the title
//   screen and main menu draw.
// - It never adds the bar to a container and never asks it to paint, so this layer draws nothing.
//   Parent Component is a guess: no call reaches the parent, and lwc widgets in this package use it.
// - Both setters answer true; the title does not branch on the result as far as the trace shows.
pub struct GProgressBar;

impl GProgressBar {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/ktf/kfc/GProgressBar",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setMaximum", "(I)Z", Self::set_maximum, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setValue", "(I)Z", Self::set_value, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("maximum", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("value", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GProgressBar::<init>({this:?})");

        jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await
    }

    async fn set_maximum(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, maximum: i32) -> JvmResult<bool> {
        tracing::debug!("com.ktf.kfc.GProgressBar::setMaximum({this:?}, {maximum})");

        jvm.put_field(&mut this, "maximum", "I", maximum).await?;
        Ok(true)
    }

    async fn set_value(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, value: i32) -> JvmResult<bool> {
        tracing::debug!("com.ktf.kfc.GProgressBar::setValue({this:?}, {value})");

        jvm.put_field(&mut this, "value", "I", value).await?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    // 83fc429f9cbe's startApp: drop the class or a setter and its boot fails before the title screen.
    #[test]
    fn g_progress_bar_takes_maximum_and_value() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let bar = jvm.new_class("com/ktf/kfc/GProgressBar", "()V", ()).await?;
            let set: bool = jvm.invoke_virtual(&bar, "com/ktf/kfc/GProgressBar", "setMaximum", "(I)Z", (8,)).await?;
            assert!(set);
            let set: bool = jvm.invoke_virtual(&bar, "com/ktf/kfc/GProgressBar", "setValue", "(I)Z", (3,)).await?;
            assert!(set);
            assert_eq!(jvm.get_field::<i32>(&bar, "maximum", "I").await?, 8);
            assert_eq!(jvm.get_field::<i32>(&bar, "value", "I").await?, 3);

            Ok(())
        })
    }
}
