use alloc::{boxed::Box, string::String, vec::Vec};
use core::{
    ops::{Deref, DerefMut},
    pin::Pin,
};

use jvm::{ClassDefinition, ClassInstance, Field, JavaValue, Jvm, Method, Result as JvmResult};
use jvm_bytecode::{ArrayClassDefinitionImpl, ClassDefinitionImpl};
use jvm_class_proto::JavaClassProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

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
        Box::pin(InheritedMethods::wrap(jvm, ClassDefinitionImpl::from_class_proto(proto, context)))
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
/// ponytail: virtual dispatch now finds an inherited method at the subclass, so its frame names
/// the subclass rather than the declaring class — cosmetic in stack traces, and the class loader
/// derived from that frame is the subclass's, which delegates to the one it replaces. Drop this
/// once the pinned crate's `invoke_special` walks superclasses itself.
#[derive(Clone, Debug)]
pub(crate) struct InheritedMethods {
    inner: ClassDefinitionImpl,
    super_class: Option<Box<dyn ClassDefinition>>,
}

impl InheritedMethods {
    pub(crate) async fn wrap(jvm: &Jvm, inner: ClassDefinitionImpl) -> JvmResult<Box<dyn ClassDefinition>> {
        // The JVM resolves the superclass right after this returns anyway; doing it first only
        // moves it earlier, through the same loader (the one calling defineClass).
        let super_class = match ClassDefinition::super_class_name(&inner) {
            Some(name) => Some(jvm.resolve_class(&name).await?.definition),
            None => None,
        };

        Ok(Box::new(Self { inner, super_class }))
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
            return Some(method);
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

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, JavaError};

    use test_utils::run_jvm_test;
    use wie_util::Result;

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
