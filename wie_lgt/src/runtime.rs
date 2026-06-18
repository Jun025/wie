pub mod init;
mod java;
mod stdlib;
mod svc_ids;
mod wipi_c;

const SVC_CATEGORY_INIT: u32 = 1;
const SVC_CATEGORY_WIPIC: u32 = 3;
const SVC_CATEGORY_STDLIB: u32 = 5;
/// Native -> platform method trampolines (the `java_load_classes` offset tables).
const SVC_CATEGORY_JAVA_TRAMPOLINE: u32 = 7;
