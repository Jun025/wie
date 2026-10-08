use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::Object;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// abstract class java.lang.ref.Reference — CLDC 1.1, which the pinned runtime does not carry.
// ponytail: the referent is held strongly, so it is never cleared by the collector. A cache keyed
// on weak references just never evicts; make `referent` weak in the GC if memory ever runs short.
pub struct Reference;

impl Reference {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "java/lang/ref/Reference",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/Object;)V", Self::init, MethodAccessFlags::empty()),
                JavaMethodProto::new("get", "()Ljava/lang/Object;", Self::get, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("clear", "()V", Self::clear, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![JavaFieldProto::new("referent", "Ljava/lang/Object;", FieldAccessFlags::PRIVATE)],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, referent: ClassInstanceRef<Object>) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "referent", "Ljava/lang/Object;", referent).await
    }

    async fn get(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Object>> {
        jvm.get_field(&this, "referent", "Ljava/lang/Object;").await
    }

    async fn clear(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        jvm.put_field(&mut this, "referent", "Ljava/lang/Object;", None).await
    }
}

// class java.lang.ref.WeakReference
pub struct WeakReference;

impl WeakReference {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "java/lang/ref/WeakReference",
            parent_class: Some("java/lang/ref/Reference"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "<init>",
                "(Ljava/lang/Object;)V",
                Self::init,
                MethodAccessFlags::PUBLIC,
            )],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, referent: ClassInstanceRef<Object>) -> JvmResult<()> {
        jvm.invoke_special(&this, "java/lang/ref/Reference", "<init>", "(Ljava/lang/Object;)V", (referent,))
            .await
    }
}
