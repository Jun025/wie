use alloc::boxed::Box;

use jvm::ClassInstance;

use wie_core_arm::ArmCore;
use wie_jvm_support::native::NativeJavaValueCodec;

use super::{array_class_instance::JavaArrayClassInstance, class_instance::JavaClassInstance};

#[derive(Clone)]
pub struct JavaValueCodec {
    core: ArmCore,
}

impl JavaValueCodec {
    pub fn new(core: &ArmCore) -> Self {
        Self { core: core.clone() }
    }
}

impl NativeJavaValueCodec for JavaValueCodec {
    fn object_from_raw(&self, raw: u32) -> Option<Box<dyn ClassInstance>> {
        let instance = JavaClassInstance::from_raw(raw, &self.core);
        // Same gate as LGT: reading the class is the liveness check, and the last frame where the
        // failure still has somewhere to go. No KTF title is known to reach it — this carrier is
        // kept in step because the codec trait is shared, not on its own evidence.
        match instance.class().and_then(|class| class.is_array()) {
            Ok(true) => Some(Box::new(JavaArrayClassInstance::from_raw(raw, &self.core))),
            Ok(false) => Some(Box::new(instance)),
            Err(error) => {
                tracing::warn!("KTF object reference {raw:#x} does not point at a live instance: {error}");
                None
            }
        }
    }

    fn object_to_raw(&self, object: &dyn ClassInstance) -> u32 {
        if let Some(instance) = object.as_any().downcast_ref::<JavaClassInstance>() {
            instance.ptr_raw
        } else {
            object.as_any().downcast_ref::<JavaArrayClassInstance>().unwrap().class_instance.ptr_raw
        }
    }
}
