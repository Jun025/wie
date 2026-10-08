use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};
use core::{
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::atomic::{AtomicU32, Ordering},
};

use jvm::{ClassDefinition, ClassInstance, Field, JavaValue, Jvm, Method, Result as JvmResult};
use jvm_bytecode::{ArrayClassDefinitionImpl, ClassDefinitionImpl};
use jvm_class_proto::JavaClassProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_backend::YieldFuture;

pub trait JvmImplementation: Clone {
    #[allow(clippy::type_complexity)]
    fn define_class_rust<'a, C, Context>(
        &'a self,
        jvm: &'a Jvm,
        proto: JavaClassProto<C>,
        context: Context,
    ) -> Pin<Box<dyn Future<Output = JvmResult<Box<dyn ClassDefinition>>> + Send + 'a>>
    // XXX we get one type is more general error if we use impl Future
    where
        C: ?Sized + 'static + Send,
        Context: Sync + Send + DerefMut + Deref<Target = C> + Clone + 'static;
    fn define_array_class(&self, jvm: &Jvm, element_type_name: &str) -> impl Future<Output = JvmResult<Box<dyn ClassDefinition>>> + Send;
}

#[derive(Clone)]
pub struct RustJavaJvmImplementation;

impl JvmImplementation for RustJavaJvmImplementation {
    fn define_class_rust<'a, C, Context>(
        &'a self,
        jvm: &'a Jvm,
        proto: JavaClassProto<C>,
        context: Context,
    ) -> Pin<Box<dyn Future<Output = JvmResult<Box<dyn ClassDefinition>>> + Send + 'a>>
    where
        C: ?Sized + 'static + Send,
        Context: Sync + Send + DerefMut + Deref<Target = C> + Clone + 'static,
    {
        Box::pin(InheritedMethods::wrap(jvm, ClassDefinitionImpl::from_class_proto(proto, context), None))
    }

    async fn define_array_class(&self, _jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>> {
        Ok(Box::new(ArrayClassDefinitionImpl::new(element_type_name)))
    }
}

/// Lets `method()` answer for methods the class inherits, so `invokespecial` resolves the way
/// JVMS §5.4.3.3 says: in the named class *and its superclasses*.
///
/// The pinned `jvm` crate's `invoke_special` asks the named class alone, so a subclass's
/// `super.getHeight()` — which javac compiles to `invokespecial` naming the direct superclass —
/// raised NoSuchMethodError whenever that superclass only inherited the method (Canvas from
/// Displayable, 2026-09-27 census). The crate comes from crates.io, so the lookup is widened here,
/// at the two places wie hands it a class definition, instead of in the crate.
///
/// Only instance methods that a subclass can inherit are widened: not `<init>`/`<clinit>` (a
/// constructor or initializer is never inherited), not `static` (`invokestatic` already walks the
/// hierarchy, and answering here would synchronize an inherited `static synchronized` method on
/// the wrong class), not `private`.
///
/// Error class differs from JVMS on malformed bytecode only: §5.4.3.3 finds a superclass's
/// `private`/`static` method and then fails with IllegalAccessError/IncompatibleClassChangeError;
/// here it is not found, so NoSuchMethodError. javac output never reaches that case.
///
/// KTF and LGT do not get this: they define classes through their own `JvmImplementation`.
///
/// ponytail: virtual dispatch now finds an inherited method at the subclass, so its frame names
/// the subclass rather than the declaring class — cosmetic in stack traces, and the class loader
/// derived from that frame is the subclass's, which delegates to the one it replaces. Drop this
/// once the pinned crate's `invoke_special` walks superclasses itself.
#[derive(Clone, Debug)]
pub(crate) struct InheritedMethods {
    inner: ClassDefinitionImpl,
    super_class: Option<Box<dyn ClassDefinition>>,
    // `Some` for a class defined from guest bytecode: its own methods yield, see `Preemptible`.
    guest_calls: Option<GuestCalls>,
}

impl InheritedMethods {
    pub(crate) async fn wrap(jvm: &Jvm, inner: ClassDefinitionImpl, guest_calls: Option<GuestCalls>) -> JvmResult<Box<dyn ClassDefinition>> {
        // The JVM resolves the superclass right after this returns anyway; doing it first only
        // moves it earlier, through the same loader (the one calling defineClass).
        let super_class = match ClassDefinition::super_class_name(&inner) {
            Some(name) => Some(jvm.resolve_class(&name).await?.definition),
            None => None,
        };

        Ok(Box::new(Self {
            inner,
            super_class,
            guest_calls,
        }))
    }
}

#[async_trait::async_trait]
impl ClassDefinition for InheritedMethods {
    fn name(&self) -> String {
        ClassDefinition::name(&self.inner)
    }

    fn super_class_name(&self) -> Option<String> {
        ClassDefinition::super_class_name(&self.inner)
    }

    fn interface_names(&self) -> Vec<String> {
        ClassDefinition::interface_names(&self.inner)
    }

    fn access_flags(&self) -> ClassAccessFlags {
        ClassDefinition::access_flags(&self.inner)
    }

    async fn instantiate(&self, jvm: &Jvm) -> JvmResult<Box<dyn ClassInstance>> {
        ClassDefinition::instantiate(&self.inner, jvm).await
    }

    async fn prepare(&self, jvm: &Jvm) -> JvmResult<()> {
        ClassDefinition::prepare(&self.inner, jvm).await
    }

    fn method(&self, name: &str, descriptor: &str, is_static: bool) -> Option<Box<dyn Method>> {
        if let Some(method) = ClassDefinition::method(&self.inner, name, descriptor, is_static) {
            return Some(match &self.guest_calls {
                Some(calls) => Box::new(Preemptible {
                    inner: method,
                    calls: calls.clone(),
                }),
                None => method,
            });
        }
        if is_static || name.starts_with('<') {
            return None;
        }

        self.super_class
            .as_ref()?
            .method(name, descriptor, false)
            .filter(|method| !method.access_flags().contains(MethodAccessFlags::PRIVATE))
    }

    fn field(&self, name: &str, descriptor: &str, is_static: bool) -> Option<Box<dyn Field>> {
        ClassDefinition::field(&self.inner, name, descriptor, is_static)
    }

    fn fields(&self) -> Vec<Box<dyn Field>> {
        ClassDefinition::fields(&self.inner)
    }

    fn get_static_field(&self, field: &dyn Field) -> JvmResult<JavaValue> {
        ClassDefinition::get_static_field(&self.inner, field)
    }

    fn put_static_field(&mut self, field: &dyn Field, value: JavaValue) -> JvmResult<()> {
        ClassDefinition::put_static_field(&mut self.inner, field, value)
    }
}

/// Guest bytecode method calls a JVM runs between two forced yields.
///
/// The pinned `jvm-bytecode` interpreter never returns `Pending` on its own: a guest thread that
/// computes without calling `Thread.sleep`/`yield`/`wait` holds the executor for as long as it
/// computes. loveme interprets Lua in `startApp` for 30 s and more without a single yield, so
/// `tick` — and with it `wie_validate --timeout` and the browser's frame — did not return
/// (`docs/report/0485` §4). The ARM core already yields every `INSTRUCTIONS_PER_YIELD`; this is
/// the same thing for the bytecode JVM, counted at method entry because the interpreter loop
/// lives in a crates.io crate and method entry is the one place wie hands it code.
///
/// A loop that never calls a method is not covered; LuaJ, and every guest seen so far, calls.
///
/// Measured, not chosen (`docs/report/0487`): loveme's ticks averaged 126 ms at 10 000, 33 ms at
/// 2 500 and 23 ms at 1 000 (load ~27, 14 ms budget), while 86 SKT/J2ME titles painted the same
/// at 1 000 as with no forced yield (median ratio 1.000). Raising it lengthens the worst tick a
/// non-yielding guest can hold; lowering it buys little more, since the yield costs one executor
/// step against ~1 000 interpreted calls.
pub(crate) const GUEST_CALLS_PER_YIELD: u32 = 1_000;

/// Guest method calls since the last forced yield, shared by every class one runtime defines.
#[derive(Clone, Debug, Default)]
pub(crate) struct GuestCalls(Arc<AtomicU32>);

#[derive(Debug)]
struct Preemptible {
    inner: Box<dyn Method>,
    calls: GuestCalls,
}

#[async_trait::async_trait]
impl Method for Preemptible {
    fn name(&self) -> String {
        self.inner.name()
    }

    fn descriptor(&self) -> String {
        self.inner.descriptor()
    }

    fn access_flags(&self) -> MethodAccessFlags {
        self.inner.access_flags()
    }

    async fn run(&self, jvm: &Jvm, args: Box<[JavaValue]>) -> JvmResult<JavaValue> {
        // A guest method could already suspend (sleep, yield, wait, a contended monitor), so every
        // caller already copes with it not finishing in one poll; this adds no new kind of suspension.
        if self.calls.0.fetch_add(1, Ordering::Relaxed) + 1 >= GUEST_CALLS_PER_YIELD {
            self.calls.0.store(0, Ordering::Relaxed);
            YieldFuture::new().await;
        }
        self.inner.run(jvm, args).await
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc, vec::Vec};
    use core::sync::atomic::{AtomicU32, Ordering};

    use jvm::{ClassInstanceRef, JavaError, runtime::JavaLangString};

    use test_utils::{TestPlatform, run_jvm_test, run_jvm_test_with_system};
    use wie_backend::YieldFuture;
    use wie_util::Result;

    use super::GUEST_CALLS_PER_YIELD;

    /// `public class Spin { public static void nop() {} public static void run(int n) { for (int i = 0; i < n; i++) nop(); } }`,
    /// assembled by hand (class version 49: no StackMapTable) — a guest that computes without
    /// ever sleeping or yielding, the shape loveme's Lua interpreter has.
    fn spin_class() -> Vec<u8> {
        fn utf8(out: &mut Vec<u8>, s: &str) {
            out.push(1);
            out.extend_from_slice(&(s.len() as u16).to_be_bytes());
            out.extend_from_slice(s.as_bytes());
        }
        fn method(out: &mut Vec<u8>, name: u16, descriptor: u16, max_stack: u16, max_locals: u16, code: &[u8]) {
            out.extend_from_slice(&[0x00, 0x09]); // public static
            out.extend_from_slice(&name.to_be_bytes());
            out.extend_from_slice(&descriptor.to_be_bytes());
            out.extend_from_slice(&[0x00, 0x01, 0x00, 11]); // one attribute: Code (#11)
            out.extend_from_slice(&(12 + code.len() as u32).to_be_bytes());
            out.extend_from_slice(&max_stack.to_be_bytes());
            out.extend_from_slice(&max_locals.to_be_bytes());
            out.extend_from_slice(&(code.len() as u32).to_be_bytes());
            out.extend_from_slice(code);
            out.extend_from_slice(&[0, 0, 0, 0]); // no exception table, no attributes
        }

        let mut out = Vec::from([0xca, 0xfe, 0xba, 0xbe, 0, 0, 0, 49, 0, 12]);
        utf8(&mut out, "Spin"); // #1
        out.extend_from_slice(&[7, 0, 1]); // #2 Class Spin
        utf8(&mut out, "java/lang/Object"); // #3
        out.extend_from_slice(&[7, 0, 3]); // #4 Class Object
        utf8(&mut out, "nop"); // #5
        utf8(&mut out, "()V"); // #6
        out.extend_from_slice(&[12, 0, 5, 0, 6]); // #7 NameAndType nop:()V
        out.extend_from_slice(&[10, 0, 2, 0, 7]); // #8 Methodref Spin.nop
        utf8(&mut out, "run"); // #9
        utf8(&mut out, "(I)V"); // #10
        utf8(&mut out, "Code"); // #11
        out.extend_from_slice(&[0x00, 0x21, 0, 2, 0, 4, 0, 0, 0, 0, 0, 2]); // public super, this, super, no interfaces/fields, 2 methods
        method(&mut out, 5, 6, 0, 0, &[0xb1]);
        #[rustfmt::skip]
        let run = [
            0x03, 0x3c,             // 0: iconst_0, istore_1
            0x1b, 0x1a,             // 2: iload_1, iload_0
            0xa2, 0x00, 0x0c,       // 4: if_icmpge 16
            0xb8, 0x00, 0x08,       // 7: invokestatic Spin.nop
            0x84, 0x01, 0x01,       // 10: iinc 1 1
            0xa7, 0xff, 0xf5,       // 13: goto 2
            0xb1,                   // 16: return
        ];
        method(&mut out, 9, 10, 2, 2, &run);
        out.extend_from_slice(&[0, 0]); // no class attributes

        out
    }

    /// A guest that never sleeps must still let the host's other tasks run: without a forced
    /// yield the whole loop runs inside one poll, `tick` does not return until it ends, and
    /// `wie_validate --timeout` and the browser's frame wait with it (loveme, `docs/report/0485`).
    #[test]
    fn a_guest_that_never_yields_is_preempted_between_method_calls() -> Result<()> {
        run_jvm_test_with_system(Box::new([]), Box::new(TestPlatform::new()), |jvm, system| async move {
            let polls = Arc::new(AtomicU32::new(0));
            let polls_clone = polls.clone();
            system.spawn(async move || {
                loop {
                    polls_clone.fetch_add(1, Ordering::Relaxed);
                    YieldFuture::new().await;
                }
            });

            let data = spin_class();
            let loader: ClassInstanceRef<()> = jvm
                .invoke_static("java/lang/ClassLoader", "getSystemClassLoader", "()Ljava/lang/ClassLoader;", ())
                .await?;
            let mut bytes = jvm.instantiate_array("B", data.len()).await?;
            jvm.store_array(&mut bytes, 0, data.iter().map(|&b| b as i8).collect::<Vec<_>>()).await?;
            let name = JavaLangString::from_rust_string(&jvm, "Spin").await?;
            let _: ClassInstanceRef<()> = jvm
                .invoke_virtual(
                    &loader,
                    "java/lang/ClassLoader",
                    "defineClass",
                    "(Ljava/lang/String;[BII)Ljava/lang/Class;",
                    (name, bytes, 0, data.len() as i32),
                )
                .await?;

            let before = polls.load(Ordering::Relaxed);
            let _: () = jvm.invoke_static("Spin", "run", "(I)V", (3 * GUEST_CALLS_PER_YIELD as i32,)).await?;
            let after = polls.load(Ordering::Relaxed);

            // 3 budgets' worth of calls is 3 forced yields; the other task runs once in each.
            assert!(after - before >= 2, "the other task ran {} times during the loop", after - before);

            Ok(())
        })
    }

    #[test]
    fn invokespecial_finds_inherited_methods_but_not_constructors() -> Result<()> {
        run_jvm_test(Box::new([]), |jvm| async move {
            let integer: ClassInstanceRef<()> = jvm.new_class("java/lang/Integer", "(I)V", (7,)).await?.into();

            // Number declares no hashCode; Object does (JVMS §5.4.3.3 walks the superclasses).
            let _: i32 = jvm.invoke_special(&integer, "java/lang/Number", "hashCode", "()I", ()).await?;

            // Integer has no ()V constructor; Number's must not stand in for it.
            let Err(JavaError::JavaException(err)) = jvm.invoke_special::<_, ()>(&integer, "java/lang/Integer", "<init>", "()V", ()).await else {
                panic!("an inherited <init> was found");
            };
            assert_eq!(err.class_definition().name(), "java/lang/NoSuchMethodError");

            Ok(())
        })
    }
}
