use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::FieldAccessFlags;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.lwc.ProgressComponent
// Kept, not drawn: this layer does not paint lwc widgets (Component::paint), so no bar shows.
// ca7fa8ade8ad builds one off the boot thread and calls only <init>(ZI)V, setValue(I)I and setMargin(II)V.
pub struct ProgressComponent;

impl ProgressComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/ProgressComponent",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(ZI)V", Self::init, Default::default()),
                JavaMethodProto::new("setValue", "(I)I", Self::set_value, Default::default()),
                JavaMethodProto::new("getValue", "()I", Self::get_value, Default::default()),
                JavaMethodProto::new("getMaxValue", "()I", Self::get_max_value, Default::default()),
                JavaMethodProto::new("setMargin", "(II)V", Self::set_margin, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("interactive", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("max", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("value", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<ProgressComponent>, interactive: bool, max: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ProgressComponent::<init>({this:?}, {interactive}, {max})");
        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "interactive", "Z", interactive).await?;
        jvm.put_field(&mut this, "max", "I", max).await?;
        Ok(())
    }

    // javadoc: below 0 becomes 0, above max becomes max; returns the value set.
    // ponytail: step is the javadoc default 1 (setStep is not implemented), so `value - value % step` is a no-op.
    async fn set_value(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<ProgressComponent>, value: i32) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.ProgressComponent::setValue({this:?}, {value})");
        let max: i32 = jvm.get_field(&this, "max", "I").await?;
        let value = value.min(max).max(0);
        jvm.put_field(&mut this, "value", "I", value).await?;
        Ok(value)
    }

    async fn get_value(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<ProgressComponent>) -> JvmResult<i32> {
        jvm.get_field(&this, "value", "I").await
    }

    async fn get_max_value(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<ProgressComponent>) -> JvmResult<i32> {
        jvm.get_field(&this, "max", "I").await
    }

    // Margins only shape the painted bar, which this layer does not paint.
    async fn set_margin(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<ProgressComponent>, top: i32, bottom: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ProgressComponent::setMargin({this:?}, {top}, {bottom})");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    /// ca7fa8ade8ad's wall after FormComponent(boolean): `NoClassDefFoundError org/kwis/msp/lwc/ProgressComponent`.
    #[test]
    fn progress_component_keeps_clamped_value() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let bar = jvm.new_class("org/kwis/msp/lwc/ProgressComponent", "(ZI)V", (false, 100)).await?;
            let max: i32 = jvm
                .invoke_virtual(&bar, "org/kwis/msp/lwc/ProgressComponent", "getMaxValue", "()I", ())
                .await?;
            assert_eq!(max, 100);
            let _: () = jvm
                .invoke_virtual(&bar, "org/kwis/msp/lwc/ProgressComponent", "setMargin", "(II)V", (2, 2))
                .await?;

            for (set, kept) in [(40, 40), (-5, 0), (250, 100)] {
                let ret: i32 = jvm
                    .invoke_virtual(&bar, "org/kwis/msp/lwc/ProgressComponent", "setValue", "(I)I", (set,))
                    .await?;
                assert_eq!(ret, kept);
                let got: i32 = jvm
                    .invoke_virtual(&bar, "org/kwis/msp/lwc/ProgressComponent", "getValue", "()I", ())
                    .await?;
                assert_eq!(got, kept);
            }

            let bg: i32 = jvm.invoke_virtual(&bar, "org/kwis/msp/lwc/Component", "getBackground", "()I", ()).await?;
            assert_eq!(bg, -1);

            Ok(())
        })
    }
}
