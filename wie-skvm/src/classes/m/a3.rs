use alloc::vec::Vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use super::V3;
use crate::mascot::{self, Affine};

const FIELDS: [&str; 12] = ["m00", "m01", "m02", "m03", "m10", "m11", "m12", "m13", "m20", "m21", "m22", "m23"];

// class m.A3 — SKT micro3D affine transform (micro3D `AffineTrans`: rotation 4096 = 1.0)
pub struct A3;

impl A3 {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "m/A3",
            parent_class: Some("java/lang/Object"),
            interfaces: alloc::vec![],
            methods: alloc::vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("ident", "()V", Self::ident, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("set", "(Lm/A3;)V", Self::set_from, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("mul", "(Lm/A3;Lm/A3;)V", Self::mul, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("trans", "(Lm/V3;Lm/V3;)V", Self::trans, MethodAccessFlags::PUBLIC),
            ],
            fields: FIELDS
                .iter()
                .map(|&name| JavaFieldProto::new(name, "I", FieldAccessFlags::PUBLIC))
                .collect::<Vec<_>>(),
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        Self::set(jvm, this, &mascot::IDENTITY).await
    }

    async fn ident(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        Self::set(jvm, this, &mascot::IDENTITY).await
    }

    async fn set_from(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, other: ClassInstanceRef<Self>) -> JvmResult<()> {
        let m = Self::get(jvm, &other).await?;
        Self::set(jvm, this, &m).await
    }

    // this = a · b
    async fn mul(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        a: ClassInstanceRef<Self>,
        b: ClassInstanceRef<Self>,
    ) -> JvmResult<()> {
        let m = mascot::mul(&Self::get(jvm, &a).await?, &Self::get(jvm, &b).await?);
        Self::set(jvm, this, &m).await
    }

    // dst = this · src (point transform, translation included)
    async fn trans(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        src: ClassInstanceRef<V3>,
        dst: ClassInstanceRef<V3>,
    ) -> JvmResult<()> {
        let v = mascot::transform(&Self::get(jvm, &this).await?, V3::get(jvm, &src).await?);
        V3::set(jvm, dst, v).await
    }

    pub async fn get(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Affine> {
        if this.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "A3 is null").await);
        }
        let mut m = [0; 12];
        for (v, name) in m.iter_mut().zip(FIELDS) {
            *v = jvm.get_field(this, name, "I").await?;
        }
        Ok(m)
    }

    pub async fn set(jvm: &Jvm, mut this: ClassInstanceRef<Self>, m: &Affine) -> JvmResult<()> {
        if this.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "A3 is null").await);
        }
        for (v, name) in m.iter().zip(FIELDS) {
            jvm.put_field(&mut this, name, "I", *v).await?;
        }
        Ok(())
    }
}
