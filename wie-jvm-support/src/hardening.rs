//! Null-argument guards that the RustJava pin does not carry.
//!
//! The engine used to depend on `Jun025/RustJava`, a fork whose whole reason for existing
//! was turning host-process panics into Java exceptions for KTF titles. P1 (2026-09-04)
//! dropped that fork and pinned `dlunch/RustJava@5b84dd1` instead — see
//! `docs/upstream-realign-verdict.md` §8-6 for why. Most of the fork's hardening exists
//! upstream at that rev; six axes do not, and three of those are the ones that matter:
//! passing `null` where a spec says "throw NullPointerException" makes the upstream code
//! dereference a null `ClassInstanceRef`, which **panics the emulator process**. A missing
//! *method* raises a catchable Java error; a missing *null guard* kills the host.
//!
//! Re-adding them does not need a fork. `JvmRuntime::find_rustjar_class` obtains the
//! `RuntimeClassProto` from `java_runtime` and hands it to the JVM itself, so wie can
//! wrap a method body on the way past. That is all this module does.
//!
//! It used to also re-add two *absent methods* — `Timer.schedule` (소울카드마스터2 hit it) and
//! `StringBuffer.insert` (미니고치) — both named in the old fork's commit log as
//! "trace-specified as method-not-found". **Slice D (2026-09-16) dropped both wie-side copies,
//! and this module now re-adds nothing**: measured against the pin's own source
//! (`rustjava-runtime-0.1.1`), `java/util/Timer` declares all **four** `schedule` forms plus
//! `cancel` (`timer.rs:23`) and `java/lang/StringBuffer` declares **twelve** `insert` overloads
//! (`string_buffer.rs:116`) — more than the **nine** in the WIPI platform library games compile
//! against (`docs/reference/AromaWIPI_classes.zip`). So the method axis is **5 of 5** and
//! **9 of 9**, closed by the pin rather than by us. The `add()` helper below is kept with no
//! caller on purpose, because the next absent overload is a `add()` call and not a redesign.
//! **Resume condition, as a number rather than "later": one (1) observed guest failure naming a
//! missing overload.** `wie_validate` reports it as a resolution error carrying the descriptor,
//! so a single trace is enough to pick the next one.
//!
//! What remains here is therefore **only the null guards** — the half that panics the host.
//!
//! Deliberately NOT covered (measured, not overlooked):
//! - Pending-thread GC roots need no port at all. The 13-row probe that produced the "six
//!   axes" list greps for `pending` — the *fork's identifier* — not for the protection, and
//!   the pin closes the same window under another name: `Thread.start` holds a
//!   `GlobalRef<Thread>` for the whole spawn callback and the collector walks
//!   `global_references` as roots. (Corrected 2026-09-04; the first version of this comment
//!   called it "impossible without a fork", which reported a risk that does not exist.)

use alloc::boxed::Box;

use jvm::{JavaError, JavaValue, Jvm};
use jvm_class_proto::{JavaMethodProto, MethodBody};
use rustjava_runtime::{Runtime, RuntimeClassProto};

/// Wraps a method body so a null in any of `args` raises NPE instead of reaching code
/// that unwraps it.
struct NullArgGuard {
    inner: Box<dyn MethodBody<JavaError, dyn Runtime>>,
    args: &'static [usize],
    message: &'static str,
}

#[async_trait::async_trait]
impl MethodBody<JavaError, dyn Runtime> for NullArgGuard {
    async fn call(&self, jvm: &Jvm, context: &mut (dyn Runtime + 'static), args: Box<[JavaValue]>) -> Result<JavaValue, JavaError> {
        for &index in self.args {
            if matches!(args.get(index), Some(JavaValue::Object(None))) {
                return Err(jvm.exception("java/lang/NullPointerException", self.message).await);
            }
        }

        self.inner.call(jvm, context, args).await
    }
}

/// `args` are indices into the *call frame*, so an instance method's `this` is 0.
fn guard(proto: &mut RuntimeClassProto, name: &str, descriptor: &str, args: &'static [usize], message: &'static str) -> bool {
    let Some(index) = proto.methods.iter().position(|x| x.name == name && x.descriptor == descriptor) else {
        // The pin moved and the method we meant to guard is gone or renamed. Say so:
        // a guard that silently stops applying is exactly how this hardening was lost
        // the first time.
        tracing::error!("hardening: {}::{name}{descriptor} not found — null guard NOT applied", proto.name);
        return false;
    };

    let old = proto.methods.remove(index);
    proto.methods.insert(
        index,
        JavaMethodProto {
            name: old.name,
            descriptor: old.descriptor,
            access_flags: old.access_flags,
            body: Box::new(NullArgGuard {
                inner: old.body,
                args,
                message,
            }),
        },
    );

    true
}

/// Adds a method the pin does not define. Refuses to shadow an existing one: if the pin grows
/// the method later, we must drop ours rather than silently win the lookup.
#[allow(dead_code)] // slice D: wie 사본이 전부 upstream 으로 흡수돼 현재 호출자 0.
// 다음 핀 이동에서 다시 필요해지는 도구라 «지우지 않는다» — 그 판단을 여기 적는다.
fn add(proto: &mut RuntimeClassProto, method: JavaMethodProto<dyn Runtime>) -> bool {
    if proto.methods.iter().any(|x| x.name == method.name && x.descriptor == method.descriptor) {
        tracing::error!(
            "hardening: {}::{}{} now exists upstream — drop the wie-side copy",
            proto.name,
            method.name,
            method.descriptor
        );
        return false;
    }

    proto.methods.push(method);

    true
}

/// Applies every guard that belongs to `proto`. Returns how many were applied, so a test
/// can assert the count rather than trust that the lookups still match.
pub fn harden(proto: &mut RuntimeClassProto) -> usize {
    match proto.name {
        // Spec: arraycopy throws NPE if src or dest is null. Upstream calls
        // `jvm.load_array(&src, ..)` straight away, and a null `ClassInstanceRef` panics
        // on deref.
        "java/lang/System" => guard(
            proto,
            "arraycopy",
            "(Ljava/lang/Object;ILjava/lang/Object;II)V",
            &[0, 2],
            "src or dest is null",
        ) as usize,

        // `new ByteArrayInputStream(null)` — the ctor immediately asks for the array
        // length.
        "java/io/ByteArrayInputStream" => guard(proto, "<init>", "([B)V", &[1], "byte array is null") as usize,

        // `sb.append((char[]) null, 0, 0)` — same shape, `load_array` on a null ref.
        // `insert` is absent from the pin entirely (미니고치 hit it) — added, not wrapped.
        // slice D (2026-09-16): the wie-side `insert(I,String)` copy was DROPPED here.
        // Upstream's rustjava-runtime now declares it itself, and `add()` says so out
        // loud ("now exists upstream — drop the wie-side copy"). The null guard on
        // `append([CII)` still attaches and is still the reachable half.
        "java/lang/StringBuffer" => guard(proto, "append", "([CII)Ljava/lang/StringBuffer;", &[1], "str is null") as usize,

        // No `java/util/Timer` arm any more: slice D removed it because the pin declares all
        // four `schedule` forms itself (see the module header). The comment that used to sit
        // here still claimed one-shot `schedule` was absent — the opposite of the measurement
        // that deleted the arm.
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, string::String, vec, vec::Vec};

    use jvm::{Array, ClassInstanceRef, JavaError};
    use rustjava_runtime::get_runtime_class_proto;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use super::harden;

    /// Every guarded/extended class must still resolve — if the pin moves and a descriptor
    /// changes, this fails instead of the hardening quietly vanishing.
    #[test]
    fn every_guard_is_actually_applied() {
        // One wrapped guard per class, and no added methods at all — slice D dropped both
        // wie-side copies because the pin declares them (module header). This said
        // "StringBuffer carries two" while asserting 1 on the line below it.
        for (name, expected) in [
            ("java/lang/System", 1),
            ("java/io/ByteArrayInputStream", 1),
            ("java/lang/StringBuffer", 1), // slice D: insert(I,String) now upstream
        ] {
            let mut proto = get_runtime_class_proto(name).unwrap();
            assert_eq!(harden(&mut proto), expected, "{name}: hardening not applied");
        }
    }

    fn exception_class(err: JavaError) -> String {
        let JavaError::JavaException(instance) = err;

        instance.class_definition().name()
    }

    /// The three null arguments the dropped fork guarded. Without the guard each of these
    /// unwraps a null `ClassInstanceRef` inside `java_runtime` and aborts the process, so
    /// "comes back as a Java exception at all" is the whole assertion.
    ///
    /// No game files are involved — `run_jvm_test` boots a bare JVM. That is deliberate:
    /// the corpus this hardening was originally validated against does not exist on this
    /// machine (docs/upstream-realign-verdict.md §8-1), so a test that needed it could
    /// never run here.
    #[test]
    fn null_arguments_raise_npe_instead_of_panicking() -> Result<()> {
        run_jvm_test(Box::new([]), |jvm| async move {
            // System.arraycopy(null, 0, null, 0, 0)
            let null: ClassInstanceRef<()> = None.into();
            let err = jvm
                .invoke_static(
                    "java/lang/System",
                    "arraycopy",
                    "(Ljava/lang/Object;ILjava/lang/Object;II)V",
                    (null.clone(), 0, null.clone(), 0, 0),
                )
                .await
                .map(|_: ()| ())
                .expect_err("arraycopy(null, .., null, ..) must throw");
            assert_eq!(exception_class(err), "java/lang/NullPointerException");

            // new ByteArrayInputStream(null)
            let null_bytes: ClassInstanceRef<Array<i8>> = None.into();
            let err = jvm
                .new_class("java/io/ByteArrayInputStream", "([B)V", (null_bytes,))
                .await
                .expect_err("new ByteArrayInputStream(null) must throw");
            assert_eq!(exception_class(err), "java/lang/NullPointerException");

            // new StringBuffer().append((char[]) null, 0, 0)
            let buffer = jvm.new_class("java/lang/StringBuffer", "()V", ()).await?;
            let null_chars: ClassInstanceRef<Array<u16>> = None.into();
            let err = jvm
                .invoke_virtual(
                    &buffer,
                    "java/lang/StringBuffer",
                    "append",
                    "([CII)Ljava/lang/StringBuffer;",
                    (null_chars, 0, 0),
                )
                .await
                .map(|_: ClassInstanceRef<()>| ())
                .expect_err("append((char[]) null, ..) must throw");
            assert_eq!(exception_class(err), "java/lang/NullPointerException");

            Ok(())
        })
    }

    /// The guard must not change behaviour for non-null arguments.
    #[test]
    fn non_null_arguments_still_work() -> Result<()> {
        run_jvm_test(Box::new([]), |jvm| async move {
            let mut src = jvm.instantiate_array("I", 3).await?;
            jvm.store_array(&mut src, 0, vec![1i32, 2, 3]).await?;
            let mut dest = jvm.instantiate_array("I", 3).await?;

            let _: () = jvm
                .invoke_static(
                    "java/lang/System",
                    "arraycopy",
                    "(Ljava/lang/Object;ILjava/lang/Object;II)V",
                    (src, 0, dest.clone(), 0, 3),
                )
                .await?;

            let copied: Vec<i32> = jvm.load_array(&mut dest, 0, 3).await?;
            assert_eq!(copied, vec![1, 2, 3]);

            Ok(())
        })
    }
}
