mod globals;
pub mod init;
mod java;
mod stdlib;
mod svc_ids;
mod wipi_c;

use wie_core_arm::ArmCore;
use wie_util::WieError;

const SVC_CATEGORY_INIT: u32 = 1;
const SVC_CATEGORY_JAVA_SYSTEM: u32 = 2;
const SVC_CATEGORY_WIPIC: u32 = 3;
const SVC_CATEGORY_JAVA: u32 = 4;
const SVC_CATEGORY_STDLIB: u32 = 5;
const SVC_CATEGORY_MISSING_JAVA_VTABLE_ENTRY: u32 = 6;

pub use java::LgtJvmSupport;

/// upstream #1534 appends the register dump here, at the failing call. This fork dumps once, for
/// every error, where the tick returns it (`LgtEmulator::tick_for`) — and its `run_function` hands
/// back the caller's registers on an error, so a dump taken here would only repeat that one.
pub fn with_arm_context(_core: &ArmCore, error: WieError) -> WieError {
    error
}
