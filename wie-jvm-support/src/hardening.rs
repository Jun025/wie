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
//! What remains here is the null guards — the half that panics the host — one replaced
//! body, `InputStreamReader.read` (`InputStreamReaderRead`, below), swapped the same way, and
//! one null the pin refuses that a handset takes (`NullTimeZoneIsDefault`).
//!
//! Deliberately NOT covered (measured, not overlooked):
//! - Pending-thread GC roots need no port at all. The 13-row probe that produced the "six
//!   axes" list greps for `pending` — the *fork's identifier* — not for the protection, and
//!   the pin closes the same window under another name: `Thread.start` holds a
//!   `GlobalRef<Thread>` for the whole spawn callback and the collector walks
//!   `global_references` as roots. (Corrected 2026-09-04; the first version of this comment
//!   called it "impossible without a fork", which reported a risk that does not exist.)

use alloc::{boxed::Box, vec, vec::Vec};

use encoding_rs::{EUC_KR, UTF_8};
use jvm::{ClassInstance, JavaChar, JavaError, JavaValue, Jvm, runtime::JavaLangString};
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

/// `Calendar.getInstance(TimeZone)` with a null zone answers the default zone's calendar — what
/// `getInstance()` returns — where the pin throws `NullPointerException: timeZone`.
/// 33f3e7669599 (KTF) passes null there from its key handler (no `TimeZone` call precedes it in
/// the trace), so every key threw and its title screen never moved. It plays on a handset, so the
/// handset's library takes the null.
/// ponytail: the default zone (GMT here) is assumed for null — a title that needs a different
/// zone would show a shifted clock, not an exception.
struct NullTimeZoneIsDefault {
    inner: Box<dyn MethodBody<JavaError, dyn Runtime>>,
}

#[async_trait::async_trait]
impl MethodBody<JavaError, dyn Runtime> for NullTimeZoneIsDefault {
    async fn call(&self, jvm: &Jvm, context: &mut (dyn Runtime + 'static), mut args: Box<[JavaValue]>) -> Result<JavaValue, JavaError> {
        if matches!(args.first(), Some(JavaValue::Object(None))) {
            let zone: jvm::ClassInstanceRef<()> = jvm
                .invoke_static("java/util/TimeZone", "getDefault", "()Ljava/util/TimeZone;", ())
                .await?;
            args[0] = zone.instance.into();
        }

        self.inner.call(jvm, context, args).await
    }
}

fn default_null_time_zone(proto: &mut RuntimeClassProto) -> bool {
    let Some(index) = proto
        .methods
        .iter()
        .position(|x| x.name == "getInstance" && x.descriptor == "(Ljava/util/TimeZone;)Ljava/util/Calendar;")
    else {
        tracing::error!("hardening: {}::getInstance(TimeZone) not found — null zone NOT defaulted", proto.name);
        return false;
    };
    let old = proto.methods.remove(index);
    proto.methods.insert(
        index,
        JavaMethodProto {
            name: old.name,
            descriptor: old.descriptor,
            access_flags: old.access_flags,
            body: Box::new(NullTimeZoneIsDefault { inner: old.body }),
        },
    );

    true
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

/// `InputStreamReader.read(char[], off, len)`, replacing the pin's body — which has two defects
/// that together break reading a text resource:
///
/// 1. **A short read every time.** It decodes one 10-byte chunk and returns (`BUF_SIZE = 10`,
///    `break` once anything is decoded): a 759-byte resource read into a 759-char buffer comes
///    back as **6** chars. The spec allows short reads, but titles read a whole text file with
///    ONE call and scan it for delimiters: 1b107b96bf4e (LGT) found none, built
///    `new String(chars, 2, -2)`, and every key on its menu threw from then on — «게임시작»
///    never started (measured 2026-09-30 by the progress census).
/// 2. **EUC-KR split across two chunks is garbled.** It holds back the last byte whenever it is
///    `>= 0x81`, which is also every TRAIL byte (0xA1..0xFE) of a complete character; the lead
///    then ends the buffer, a fresh decoder consumes it, and the character comes out as U+FFFD.
///
/// So this reads until `len` chars or until more would block (`available() == 0` once something
/// is read — the JDK's own `StreamDecoder` rule, so a live stream still returns what it has),
/// and holds back only a genuinely incomplete trailing character. It keeps the pin's fields
/// (`readBuf`/`writeBuf` carry the bytes and chars left over), so `ready()` and `close()` —
/// still the pin's — see the same state.
struct InputStreamReaderRead;

/// Length of the prefix of `bytes` that ends on a character boundary. `bytes` always starts on
/// one (only whole characters are ever consumed), so EUC-KR can be walked from the front.
fn complete_prefix(charset: &str, bytes: &[u8]) -> usize {
    if charset == "UTF-8" {
        let Some(mut lead) = bytes.len().checked_sub(1) else { return 0 };
        while lead > 0 && bytes[lead] & 0xc0 == 0x80 {
            lead -= 1;
        }
        let need = match bytes[lead] {
            0xc0..=0xdf => 2,
            0xe0..=0xef => 3,
            0xf0..=0xf7 => 4,
            _ => 1,
        };
        if bytes.len() - lead < need { lead } else { bytes.len() }
    } else {
        let mut i = 0;
        while i < bytes.len() {
            let width = if bytes[i] >= 0x81 { 2 } else { 1 };
            if i + width > bytes.len() {
                break;
            }
            i += width;
        }
        i
    }
}

#[async_trait::async_trait]
impl MethodBody<JavaError, dyn Runtime> for InputStreamReaderRead {
    async fn call(&self, jvm: &Jvm, _: &mut (dyn Runtime + 'static), args: Box<[JavaValue]>) -> Result<JavaValue, JavaError> {
        let [
            JavaValue::Object(Some(this)),
            JavaValue::Object(buf),
            JavaValue::Int(offset),
            JavaValue::Int(length),
        ] = &*args
        else {
            unreachable!("read([CII)I is called with (this, char[], int, int)");
        };
        let (mut this, offset, length) = (this.clone(), *offset, *length);
        let Some(mut buf) = buf.clone() else {
            return Err(jvm.exception("java/lang/NullPointerException", "buffer is null").await);
        };
        if offset < 0 || length < 0 || offset > jvm.array_length(&buf).await? as i32 - length {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "Invalid offset or length").await);
        }
        if length == 0 {
            return Ok(JavaValue::Int(0));
        }

        let charset = JavaLangString::to_rust_string(jvm, &jvm.get_field(&this, "charset", "Ljava/lang/String;").await?).await?;
        let input: Box<dyn ClassInstance> = jvm.get_field(&this, "in", "Ljava/io/InputStream;").await?;
        let mut write_buf: Box<dyn ClassInstance> = jvm.get_field(&this, "writeBuf", "[C").await?;
        let pending: i32 = jvm.get_field(&this, "writeBufSize", "I").await?;
        let mut chars: Vec<JavaChar> = jvm.load_array(&write_buf, 0, pending as usize).await?;
        let mut read_buf: Box<dyn ClassInstance> = jvm.get_field(&this, "readBuf", "[B").await?;
        let carried: i32 = jvm.get_field(&this, "readBufSize", "I").await?;
        let mut bytes: Vec<u8> = jvm
            .load_array::<i8>(&read_buf, 0, carried as usize)
            .await?
            .into_iter()
            .map(|b| b as u8)
            .collect();
        let mut end_of_input: bool = jvm.get_field(&this, "endOfInput", "Z").await?;

        while (chars.len() as i32) < length && !end_of_input {
            if !chars.is_empty()
                && jvm
                    .invoke_virtual::<_, i32>(&input, "java/io/InputStream", "available", "()I", ())
                    .await?
                    <= 0
            {
                break;
            }
            // A character is at least one byte, so asking for the chars still wanted never
            // decodes past `len` by more than the one character a carried lead completes.
            let want = (length as usize - chars.len()).max(1);
            let temp = jvm.instantiate_array("B", want).await?;
            let read: i32 = jvm
                .invoke_virtual(&input, "java/io/InputStream", "read", "([BII)I", (temp.clone(), 0, want as i32))
                .await?;
            if read < 0 {
                end_of_input = true;
            } else {
                bytes.extend(jvm.load_array::<i8>(&temp, 0, read as usize).await?.into_iter().map(|b| b as u8));
            }
            let complete = if end_of_input { bytes.len() } else { complete_prefix(&charset, &bytes) };
            let mut decoder = if charset == "UTF-8" { UTF_8 } else { EUC_KR }.new_decoder_without_bom_handling();
            let mut decoded = vec![0u16; decoder.max_utf16_buffer_length(complete).unwrap_or(complete * 2)];
            let (_, _, wrote, _) = decoder.decode_to_utf16(&bytes[..complete], &mut decoded, true);
            chars.extend_from_slice(&decoded[..wrote]);
            bytes.drain(..complete);
            if read == 0 {
                break;
            }
        }

        if chars.is_empty() && end_of_input {
            return Ok(JavaValue::Int(-1));
        }
        let out = chars.len().min(length as usize);
        jvm.store_array(&mut buf, offset as usize, chars[..out].to_vec()).await?;

        let rest = &chars[out..];
        if rest.len() > jvm.array_length(&write_buf).await? {
            write_buf = jvm.instantiate_array("C", rest.len()).await?;
            jvm.put_field(&mut this, "writeBuf", "[C", write_buf.clone()).await?;
        }
        jvm.store_array(&mut write_buf, 0, rest.to_vec()).await?;
        jvm.put_field(&mut this, "writeBufSize", "I", rest.len() as i32).await?;
        if bytes.len() > jvm.array_length(&read_buf).await? {
            read_buf = jvm.instantiate_array("B", bytes.len()).await?;
            jvm.put_field(&mut this, "readBuf", "[B", read_buf.clone()).await?;
        }
        jvm.store_array(&mut read_buf, 0, bytes.iter().map(|&b| b as i8).collect::<Vec<_>>())
            .await?;
        jvm.put_field(&mut this, "readBufSize", "I", bytes.len() as i32).await?;
        jvm.put_field(&mut this, "endOfInput", "Z", end_of_input).await?;

        Ok(JavaValue::Int(out as i32))
    }
}

fn replace_reader_read(proto: &mut RuntimeClassProto) -> bool {
    let Some(method) = proto.methods.iter_mut().find(|x| x.name == "read" && x.descriptor == "([CII)I") else {
        tracing::error!("hardening: {}::read([CII)I not found — reader read NOT replaced", proto.name);
        return false;
    };
    method.body = Box::new(InputStreamReaderRead);

    true
}

/// `new InputStreamReader(in, "US-ASCII")` reads as UTF-8 — an exact superset for every byte ASCII
/// defines. The pin knows only UTF-8 and EUC-KR and throws `UnsupportedEncodingException` for the
/// rest; CLDC handsets take US-ASCII, and j2me-2048 reads its English locale file with it, caught the
/// exception as an `IOException` and called `notifyDestroyed` on the first key (2026-10-08).
/// ponytail: only the ASCII names — ISO-8859-1 differs from both decoders above 0x7F, so it still throws.
struct AsciiReadsAsUtf8 {
    inner: Box<dyn MethodBody<JavaError, dyn Runtime>>,
}

#[async_trait::async_trait]
impl MethodBody<JavaError, dyn Runtime> for AsciiReadsAsUtf8 {
    async fn call(&self, jvm: &Jvm, context: &mut (dyn Runtime + 'static), mut args: Box<[JavaValue]>) -> Result<JavaValue, JavaError> {
        if let Some(JavaValue::Object(Some(charset))) = args.get(2) {
            let name = JavaLangString::to_rust_string(jvm, charset).await?;
            if ["US-ASCII", "ASCII", "US_ASCII"].iter().any(|x| name.eq_ignore_ascii_case(x)) {
                args[2] = JavaValue::Object(Some(JavaLangString::from_rust_string(jvm, "UTF-8").await?));
            }
        }

        self.inner.call(jvm, context, args).await
    }
}

fn ascii_reads_as_utf8(proto: &mut RuntimeClassProto) -> bool {
    let Some(index) = proto
        .methods
        .iter()
        .position(|x| x.name == "<init>" && x.descriptor == "(Ljava/io/InputStream;Ljava/lang/String;)V")
    else {
        tracing::error!("hardening: {}::<init>(InputStream, String) not found — ASCII NOT mapped", proto.name);
        return false;
    };
    let old = proto.methods.remove(index);
    proto.methods.insert(
        index,
        JavaMethodProto {
            name: old.name,
            descriptor: old.descriptor,
            access_flags: old.access_flags,
            body: Box::new(AsciiReadsAsUtf8 { inner: old.body }),
        },
    );

    true
}

/// `Class.getResourceAsStream` falls back to the system class loader — the MIDlet jar — when the
/// class's own loader finds nothing. The pin asks the class's loader only, and a platform class's
/// loader is the bootstrap one, which has no jar resources: `Image.class.getResourceAsStream("/x.png")`
/// — the idiom of Nokia's example `ImageHelper`, copied into open-source MIDP games — came back null
/// and the game died in `Image.createImage(null)` before its first paint (j2me-lines, 2026-10-08).
/// On a handset every class reads the one jar, so the fallback is what the guest expects.
struct ResourceFromMidletJar {
    inner: Box<dyn MethodBody<JavaError, dyn Runtime>>,
}

#[async_trait::async_trait]
impl MethodBody<JavaError, dyn Runtime> for ResourceFromMidletJar {
    async fn call(&self, jvm: &Jvm, context: &mut (dyn Runtime + 'static), args: Box<[JavaValue]>) -> Result<JavaValue, JavaError> {
        let name = args.get(1).cloned();
        let found = self.inner.call(jvm, context, args).await?;
        let (JavaValue::Object(None), Some(JavaValue::Object(Some(name)))) = (&found, name) else {
            return Ok(found);
        };

        let loader: Box<dyn ClassInstance> = jvm
            .invoke_static("java/lang/ClassLoader", "getSystemClassLoader", "()Ljava/lang/ClassLoader;", ())
            .await?;
        let stream: Option<Box<dyn ClassInstance>> = jvm
            .invoke_virtual(
                &loader,
                "java/lang/ClassLoader",
                "getResourceAsStream",
                "(Ljava/lang/String;)Ljava/io/InputStream;",
                (name,),
            )
            .await?;

        Ok(JavaValue::Object(stream))
    }
}

fn resource_from_midlet_jar(proto: &mut RuntimeClassProto) -> bool {
    let Some(index) = proto
        .methods
        .iter()
        .position(|x| x.name == "getResourceAsStream" && x.descriptor == "(Ljava/lang/String;)Ljava/io/InputStream;")
    else {
        tracing::error!("hardening: {}::getResourceAsStream not found — jar fallback NOT applied", proto.name);
        return false;
    };
    let old = proto.methods.remove(index);
    proto.methods.insert(
        index,
        JavaMethodProto {
            name: old.name,
            descriptor: old.descriptor,
            access_flags: old.access_flags,
            body: Box::new(ResourceFromMidletJar { inner: old.body }),
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

        // `new String((char[]) null)` — `([C)V`/`([B)V` ask for the array length first, the
        // partial and charset forms `load_array` straight away; either derefs the null ref.
        // d552e095ddcf (KTF) panicked here 2 runs in 4 (`String::init_with_char_array`,
        // docs/report/0030 §10-6-b). All six array-taking constructors, not just the one seen.
        "java/lang/String" => [
            "([C)V",
            "([CII)V",
            "([B)V",
            "([BII)V",
            "([BLjava/lang/String;)V",
            "([BIILjava/lang/String;)V",
        ]
        .into_iter()
        .filter(|descriptor| guard(proto, "<init>", descriptor, &[1], "array is null"))
        .count(),

        "java/io/InputStreamReader" => replace_reader_read(proto) as usize + ascii_reads_as_utf8(proto) as usize,

        "java/util/Calendar" => default_null_time_zone(proto) as usize,

        "java/lang/Class" => resource_from_midlet_jar(proto) as usize,

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
            ("java/lang/StringBuffer", 1),    // slice D: insert(I,String) now upstream
            ("java/lang/String", 6),          // every array-taking <init>
            ("java/io/InputStreamReader", 2), // read body + ASCII charset
            ("java/util/Calendar", 1),
            ("java/lang/Class", 1),
        ] {
            let mut proto = get_runtime_class_proto(name).unwrap();
            assert_eq!(harden(&mut proto), expected, "{name}: hardening not applied");
        }
    }

    fn exception_class(err: JavaError) -> String {
        let JavaError::JavaException(instance) = err;

        instance.class_definition().name()
    }

    /// The three null arguments the dropped fork guarded, plus `String`'s array constructors
    /// (d552e095ddcf, 2026-09-28). Without the guard each of these
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

            // new String((char[]) null) — d552e095ddcf's panic — and new String((byte[]) null)
            let err = jvm
                .new_class("java/lang/String", "([C)V", (None.into(),) as (ClassInstanceRef<Array<u16>>,))
                .await
                .expect_err("new String((char[]) null) must throw");
            assert_eq!(exception_class(err), "java/lang/NullPointerException");
            let null_bytes: ClassInstanceRef<Array<i8>> = None.into();
            let err = jvm
                .new_class("java/lang/String", "([B)V", (null_bytes,))
                .await
                .expect_err("new String((byte[]) null) must throw");
            assert_eq!(exception_class(err), "java/lang/NullPointerException");

            Ok(())
        })
    }

    /// 33f3e7669599 calls `Calendar.getInstance(null)` on every key; the pin threw NPE there.
    #[test]
    fn calendar_get_instance_with_a_null_zone_uses_the_default_zone() -> Result<()> {
        run_jvm_test(Box::new([]), |jvm| async move {
            let null: ClassInstanceRef<()> = None.into();
            let calendar: ClassInstanceRef<()> = jvm
                .invoke_static("java/util/Calendar", "getInstance", "(Ljava/util/TimeZone;)Ljava/util/Calendar;", (null,))
                .await?;
            let zone: ClassInstanceRef<()> = jvm
                .invoke_virtual(&calendar, "java/util/Calendar", "getTimeZone", "()Ljava/util/TimeZone;", ())
                .await?;
            assert!(!zone.is_null());

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

#[cfg(test)]
mod reader_read_tests {
    use alloc::{boxed::Box, vec, vec::Vec};

    use jvm::{Array, ClassInstanceRef, runtime::JavaLangString};

    use test_utils::run_jvm_test;
    use wie_util::Result;

    /// One `read(char[], 0, n)` over a whole resource returns the whole resource — the call
    /// 1b107b96bf4e (LGT) makes on a 759-byte text file. Upstream returns 6 of 759 and
    /// this fails with `left: 6`. The reader is then drained and says so with -1. (Its requests
    /// end on line boundaries, so no character is split here — that is the next test's job.)
    #[test]
    fn one_read_fills_the_buffer_from_a_resource_stream() -> Result<()> {
        run_jvm_test(Box::new([]), |jvm| async move {
            let mut text = Vec::new();
            for i in 0..60 {
                text.extend_from_slice(b"line ");
                text.extend_from_slice(&[0xb0, 0xa1, 0xb3, 0xaa]); // «가나» in EUC-KR
                text.push(b'0' + (i % 10) as u8);
                text.push(b'\n');
            }
            let expected = 60 * 9;

            let mut bytes = jvm.instantiate_array("B", text.len()).await?;
            jvm.store_array(&mut bytes, 0, text.iter().map(|&b| b as i8).collect::<Vec<_>>()).await?;
            let stream = jvm.new_class("java/io/ByteArrayInputStream", "([B)V", (bytes,)).await?;
            let charset = JavaLangString::from_rust_string(&jvm, "EUC-KR").await?;
            let reader = jvm
                .new_class(
                    "java/io/InputStreamReader",
                    "(Ljava/io/InputStream;Ljava/lang/String;)V",
                    (stream, charset),
                )
                .await?;

            let chars: ClassInstanceRef<Array<u16>> = jvm.instantiate_array("C", expected + 10).await?.into();
            let read: i32 = jvm
                .invoke_virtual(&reader, "java/io/Reader", "read", "([CII)I", (chars.clone(), 0, expected as i32 + 10))
                .await?;
            assert_eq!(read, expected as i32);

            let got: Vec<u16> = jvm.load_array(&chars, 0, expected).await?;
            let want: Vec<u16> = (0..60)
                .flat_map(|i| "line 가나".encode_utf16().chain([u16::from(b'0' + (i % 10) as u8), 10]))
                .collect();
            assert_eq!(got, want);

            let eof: i32 = jvm.invoke_virtual(&reader, "java/io/Reader", "read", "([CII)I", (chars, 0, 10)).await?;
            assert_eq!(eof, -1);

            Ok(())
        })
    }

    /// `new InputStreamReader(in, "US-ASCII")` opens and reads — the pin threw
    /// `UnsupportedEncodingException` and j2me-2048 quit on it.
    #[test]
    fn us_ascii_reader_opens_and_reads() -> Result<()> {
        run_jvm_test(Box::new([]), |jvm| async move {
            let mut bytes = jvm.instantiate_array("B", 2).await?;
            jvm.store_array(&mut bytes, 0, vec![b'h' as i8, b'i' as i8]).await?;
            let stream = jvm.new_class("java/io/ByteArrayInputStream", "([B)V", (bytes,)).await?;
            let charset = JavaLangString::from_rust_string(&jvm, "us-ascii").await?;
            let reader = jvm
                .new_class(
                    "java/io/InputStreamReader",
                    "(Ljava/io/InputStream;Ljava/lang/String;)V",
                    (stream, charset),
                )
                .await?;

            let chars: ClassInstanceRef<Array<u16>> = jvm.instantiate_array("C", 4).await?.into();
            let read: i32 = jvm
                .invoke_virtual(&reader, "java/io/Reader", "read", "([CII)I", (chars.clone(), 0, 4))
                .await?;
            assert_eq!(read, 2);
            assert_eq!(jvm.load_array::<u16>(&chars, 0, 2).await?, [b'h' as u16, b'i' as u16]);

            Ok(())
        })
    }

    /// A two-byte EUC-KR character split across two requests comes back whole. One ASCII byte in
    /// front puts every 2-char request's byte boundary INSIDE a Hangul syllable (x | B0 A1 | …),
    /// so the lead byte has to be carried to the next request. Without the carry
    /// (`complete_prefix` bypassed) the lead is decoded alone and this reads U+FFFD.
    #[test]
    fn a_character_split_across_requests_is_carried_whole() -> Result<()> {
        run_jvm_test(Box::new([]), |jvm| async move {
            let mut text = vec![b'x'];
            for _ in 0..8 {
                text.extend_from_slice(&[0xb0, 0xa1, 0xb3, 0xaa]); // «가나» in EUC-KR
            }

            let mut bytes = jvm.instantiate_array("B", text.len()).await?;
            jvm.store_array(&mut bytes, 0, text.iter().map(|&b| b as i8).collect::<Vec<_>>()).await?;
            let stream = jvm.new_class("java/io/ByteArrayInputStream", "([B)V", (bytes,)).await?;
            let charset = JavaLangString::from_rust_string(&jvm, "EUC-KR").await?;
            let reader = jvm
                .new_class(
                    "java/io/InputStreamReader",
                    "(Ljava/io/InputStream;Ljava/lang/String;)V",
                    (stream, charset),
                )
                .await?;

            let chars: ClassInstanceRef<Array<u16>> = jvm.instantiate_array("C", 2).await?.into();
            let mut got = Vec::new();
            loop {
                let read: i32 = jvm
                    .invoke_virtual(&reader, "java/io/Reader", "read", "([CII)I", (chars.clone(), 0, 2))
                    .await?;
                if read < 0 {
                    break;
                }
                assert!(read > 0, "a read of 2 chars returned 0 before the end");
                got.extend(jvm.load_array::<u16>(&chars, 0, read as usize).await?);
            }
            let want: Vec<u16> = "x가나가나가나가나가나가나가나가나".encode_utf16().collect();
            assert_eq!(
                alloc::string::String::from_utf16_lossy(&got),
                alloc::string::String::from_utf16_lossy(&want)
            );

            Ok(())
        })
    }
}
