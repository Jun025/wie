mod init;
mod java;
mod relocated;
mod svc_ids;
mod wipi_c;

const SVC_CATEGORY_INIT: u32 = 1;
const SVC_CATEGORY_JAVA_INTERFACE: u32 = 2;
const SVC_CATEGORY_WIPIC: u32 = 3;
const SVC_CATEGORY_JAVA: u32 = 4;
const SVC_CATEGORY_RELOCATED: u32 = 5;

pub use self::java::jvm_support::{KtfJvmSupport, KtfJvmThreadContext};
