use alloc::{boxed::Box, format, string::String};
use core::fmt::{self, Debug, Formatter};

use jvm::{ArrayClassDefinition, ClassDefinition, ClassInstance, JavaType, Jvm, Result as JvmResult};

use wie_core_arm::ArmCore;
use wie_jvm_support::native::array_element_size;
use wie_util::Result;

use crate::runtime::java::JavaSvcFunctions;

use super::{JavaArrayClassInstance, JavaClassDefinition};

#[derive(Clone)]
pub struct JavaArrayClassDefinition {
    pub class: JavaClassDefinition,
    core: ArmCore,
}

impl JavaArrayClassDefinition {
    pub async fn new(core: &mut ArmCore, jvm: &Jvm, element_type_name: &str, functions: JavaSvcFunctions) -> Result<Self> {
        let class = JavaClassDefinition::new_array(core, jvm, &format!("[{element_type_name}"), functions).await?;
        Ok(Self { class, core: core.clone() })
    }

    pub fn from_class(class: JavaClassDefinition, core: &ArmCore) -> Self {
        Self { class, core: core.clone() }
    }

    fn element_type_descriptor(&self) -> String {
        let class_name = ClassDefinition::name(&self.class);
        class_name[1..].into()
    }

    pub fn element_size(&self) -> usize {
        array_element_size(&self.element_type())
    }

    pub fn element_type(&self) -> JavaType {
        JavaType::parse(&self.element_type_descriptor())
    }
}

#[async_trait::async_trait]
impl ArrayClassDefinition for JavaArrayClassDefinition {
    fn element_type_name(&self) -> String {
        self.element_type_descriptor()
    }

    async fn instantiate_array(&self, jvm: &Jvm, length: usize) -> JvmResult<Box<dyn ClassInstance>> {
        match JavaArrayClassInstance::new(&mut self.core.clone(), self, length) {
            Ok(instance) => Ok(Box::new(instance)),
            Err(error) => Err(super::host_error(jvm, &self.core, "net/wie/WieError", &format!("Failed to instantiate array: {error}")).await),
        }
    }
}

impl Debug for JavaArrayClassDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("JavaArrayClassDefinition").field("class", &self.class).finish()
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc};
    use core::sync::atomic::{AtomicBool, Ordering};

    use jvm::JavaError;

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_core_arm::Allocator;
    use wie_jvm_support::JvmSupport;
    use wie_util::{Result, WieError};

    use crate::runtime::java::{
        exception,
        jvm_support::{LgtJvmSupport, tests::init_jvm},
    };

    #[test]
    fn exhausted_heap_ends_array_instantiation_with_a_host_error() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let (jvm, mut core, _) = init_jvm(&system_clone).await?;

            // With room left for the error, a failed allocation still throws it into the guest.
            let Err(JavaError::JavaException(error)) = jvm.instantiate_array("I", 0x1000_0000).await else {
                panic!("an array larger than the heap must fail");
            };
            assert_ne!(LgtJvmSupport::class_instance_raw(&*error), 0);
            assert!(!exception::host_error_unbuildable(&core)?);

            // 놈3: the array and its error's message both find their allocator full, so building the
            // error fails the way the array did. Before, that recursed until the stack overflowed.
            // The int[256] goes to the list half (over 512 bytes), filled by halving; the message's
            // char[] to the 128-byte bucket.
            let mut size = 0x800_0000;
            while size > 512 {
                if Allocator::alloc(&mut core, size).is_err() {
                    size /= 2;
                }
            }
            while Allocator::alloc(&mut core, 128).is_ok() {}
            let Err(error) = jvm.instantiate_array("I", 256).await else {
                panic!("the heap is full");
            };
            assert!(exception::host_error_unbuildable(&core)?);
            assert!(matches!(exception::unwind(&mut core, 0), Err(WieError::FatalError(_))));
            assert!(matches!(JvmSupport::to_wie_err(&jvm, error).await, WieError::FatalError(_)));

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }
}
