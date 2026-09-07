//! `Image.createImage(String)` on a name the classloader cannot resolve — the message must still
//! name the resource. This locks the NAME, not the wording.
//!
//! Why only half a lock, and why this half. `wie_j2me/tests/test_boot.rs` already locks the TYPE of
//! both createImage failure branches (IOException / IllegalArgumentException) through the markers
//! `scripts/make-draw-fixture.mjs` prints from narrow catch handlers. What no test looked at was the
//! message. Measured 2026-09-07: there are exactly TWO such messages, and nothing reads either one —
//! every hit outside the two `jvm.exception(` call sites in
//! `wie_midp/src/classes/javax/microedition/lcdui/image.rs` is prose (this ledger, `STATE.md`,
//! `docs/`, comments), so neither is machine-depended-on today.
//!
//! The two are not the same kind of string, which is the whole finding:
//!
//!   * `format!("Resource not found: {name}")` INTERPOLATES the resource name. That name is the
//!     one datum that makes the error actionable, and it has three times been the diagnostic that
//!     told a human what broke — `docs/upstream-realign-verdict.md:796`, `STATE.md:349`, and the E1
//!     probe in `docs/report/0047--2026-09-06--wie-system-class-loader-createimage-fixture.md` all
//!     identify a regression by reading `/wie-img.png` out of this message. Dropping `{name}` would
//!     keep the type lock green while silently deleting that. Hence this test.
//!   * `"Failed to decode image"` is a constant that restates its own exception type, so locking it
//!     could only ever fire on a wording edit — cost with no benefit, given the type is already
//!     locked. DELIBERATELY NOT LOCKED. If you came here to ask "should we lock that one too", the
//!     answer measured on 2026-09-07 is no, and `docs/worklog/2026-09-07-createimage-message-lock.json`
//!     records what was weighed.
//!
//! So: assert the name is present, assert nothing about the prose around it. Rewording the message
//! is free; deleting the interpolation is not.
//!
//! Host-side rather than in the fixture on purpose — `test_utils::run_jvm_test` already exists and
//! reaches the branch with no jar loaded, so this costs one file instead of new guest bytecode plus
//! a fixture rebuild. One test per file, matching `wie_jvm_support/tests/`. No game files.

use jvm::{ClassInstanceRef, JavaError, runtime::JavaLangString};

use test_utils::run_jvm_test;
use wie_util::Result;

/// The name this test asks for. Absent from the (empty) classpath `run_jvm_test` builds, which is
/// exactly the `None =>` arm — no jar is loaded, so the lookup cannot succeed.
const ABSENT: &str = "/wie-absent-from-classpath.png";

#[test]
fn create_image_missing_name_message_carries_the_name() -> Result<()> {
    run_jvm_test(Box::new([wie_midp::get_protos().into()]), |jvm| async move {
        let name = JavaLangString::from_rust_string(&jvm, ABSENT).await?;
        let err = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(Ljava/lang/String;)Ljavax/microedition/lcdui/Image;",
                (name,),
            )
            .await
            .map(|_: ClassInstanceRef<()>| ())
            .expect_err("createImage on a name absent from the classpath must throw, not return");

        let JavaError::JavaException(instance) = err;

        // The type, so a message assertion can never be the only thing standing here: if the branch
        // starts throwing something else, this fails before the message is even read.
        assert_eq!(
            instance.class_definition().name(),
            "java/io/IOException",
            "the missing-resource branch must stay java.io.IOException"
        );

        let message: ClassInstanceRef<jvm::runtime::JavaLangString> = jvm.invoke_virtual(&instance, "getMessage", "()Ljava/lang/String;", ()).await?;
        let message = JavaLangString::to_rust_string(&jvm, &message).await?;

        // The lock: the NAME, not the sentence around it. Reword freely; do not drop the `{name}`.
        assert!(
            message.contains(ABSENT),
            "the missing-resource message no longer names the resource it could not find \
             (got {message:?}, expected it to contain {ABSENT:?}). Rewording is fine — dropping \
             the interpolated name is not; see this file's header."
        );

        Ok(())
    })
}
