use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};

use spin::Mutex;

use wie_core_arm::{ArmCore, JumpTo, RegisteredFunction, SvcId};
use wie_util::{Result, WieError};

use crate::runtime::SVC_CATEGORY_JAVA;

pub mod interface;
pub mod jvm_support;

pub type JavaSvcFunctions = Arc<Mutex<BTreeMap<u32, Arc<Box<dyn RegisteredFunction>>>>>;

async fn handle_java_svc(core: &mut ArmCore, svc_functions: &mut JavaSvcFunctions, id: SvcId) -> Result<JumpTo> {
    let function = {
        let svc_functions = svc_functions.lock();
        svc_functions
            .get(&id.0)
            .cloned()
            .ok_or_else(|| WieError::FatalError(alloc::format!("Unknown KTF Java SVC id {:#x}", id.0)))?
    };

    function.call(core).await?;

    Ok(resume_here(core))
}

/// Where the function a category handler ran left the guest. That function has already written `pc`,
/// and it is not always `lr`: a relocated image's throw is caught in place (`JavaMethod::handle_exception`).
/// A category handler returning `()` would write `lr` over it.
pub(crate) fn resume_here(core: &ArmCore) -> JumpTo {
    let context = core.save_context();

    JumpTo(context.pc | ((context.cpsr >> 5) & 1))
}

pub fn register_java_svc_handler(core: &mut ArmCore, svc_functions: &JavaSvcFunctions) -> Result<()> {
    core.register_svc_handler(SVC_CATEGORY_JAVA, handle_java_svc, svc_functions)
}
