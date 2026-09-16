//! The four classes KTF's `MExe_init` preloads resolve, and resolve to the *runtime's* copy.
//!
//! `wie_wipi_java` registered its own `java/io/InterruptedIOException`,
//! `java/io/UnsupportedEncodingException`, `java/lang/VirtualMachineError` and
//! `java/lang/OutOfMemoryError` from 2026-07-02 (`5603a7f9`): the fork pinned then did not carry
//! them, and `MExe_init` preloads all four through `load_java_class`, aborting the guest if one
//! does not resolve. The pin moved to `dlunch/RustJava@5b84dd1` on 2026-09-04 (`1762a32c`), whose
//! `java_runtime/src/loader.rs` registers all four, and `JvmSupport::new_jvm` builds
//! `java.class.path` as `RT_RUSTJAR : WIE_RUSTJAR : <jar>` — runtime first. The copies were
//! therefore unreachable and were deleted.
//!
//! This test runs with the real `get_protos()` loaded, so it asserts both halves:
//!
//! 1. **They resolve** — what the preload needs. Nothing else in the suite covers that; the
//!    corpus title that originally showed it is not on this machine.
//! 2. **They are the runtime's, not a wie copy** — `VirtualMachineError` is `ABSTRACT` upstream
//!    and was *not* in the deleted copy, so `new_class` on it throws `InstantiationError` exactly
//!    when the runtime's definition wins. Measured both ways on 2026-09-16: with the copies still
//!    present the throw is identical, which is what proved they were already shadowed.
//!
//! Re-adding a wie-side copy of any of these reddens this file. No game files.

use jvm::JavaError;

use test_utils::run_jvm_test;
use wie_util::Result;

/// Resolve and instantiate; these three are concrete on both sides.
const CONCRETE: [&str; 3] = [
    "java/io/InterruptedIOException",
    "java/io/UnsupportedEncodingException",
    "java/lang/OutOfMemoryError",
];

fn exception_class(err: JavaError) -> String {
    let JavaError::JavaException(instance) = err;

    instance.class_definition().name()
}

#[test]
fn preload_classes_come_from_the_runtime() -> Result<()> {
    run_jvm_test(Box::new([Box::new(wie_wipi_java::get_protos()) as Box<[_]>]), |jvm| async move {
        for name in CONCRETE {
            let instance = jvm.new_class(name, "()V", ()).await?;
            assert_eq!(instance.class_definition().name(), name);

            // `new_class` would also pass on a same-named class that is not a throwable, which is
            // not what the preload needs.
            let _: () = jvm
                .invoke_virtual(&instance, "printStackTrace", "()V", ())
                .await
                .unwrap_or_else(|_| panic!("{name} does not behave as a Throwable"));
        }

        // The discriminator: abstract in the runtime, concrete in the copy that used to sit in
        // this crate. A throw here means the runtime's definition is the one being resolved.
        let err = jvm
            .new_class("java/lang/VirtualMachineError", "()V", ())
            .await
            .map(|_| ())
            .expect_err("java/lang/VirtualMachineError is ABSTRACT in the pinned runtime");
        assert_eq!(exception_class(err), "java/lang/InstantiationError");

        Ok(())
    })
}
