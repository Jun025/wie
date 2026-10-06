use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class m.V3 — SKT micro3D vector (`x`, `y`, `z` read and written directly by games)
pub struct V3;

impl V3 {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "m/V3",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("<init>", "(III)V", Self::init_with_xyz, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("x", "I", FieldAccessFlags::PUBLIC),
                JavaFieldProto::new("y", "I", FieldAccessFlags::PUBLIC),
                JavaFieldProto::new("z", "I", FieldAccessFlags::PUBLIC),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
    }

    async fn init_with_xyz(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, z: i32) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        Self::set(jvm, this, [x, y, z]).await
    }

    pub async fn get(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<[i32; 3]> {
        if this.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "V3 is null").await);
        }
        Ok([
            jvm.get_field(this, "x", "I").await?,
            jvm.get_field(this, "y", "I").await?,
            jvm.get_field(this, "z", "I").await?,
        ])
    }

    pub async fn set(jvm: &Jvm, mut this: ClassInstanceRef<Self>, v: [i32; 3]) -> JvmResult<()> {
        if this.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "V3 is null").await);
        }
        jvm.put_field(&mut this, "x", "I", v[0]).await?;
        jvm.put_field(&mut this, "y", "I", v[1]).await?;
        jvm.put_field(&mut this, "z", "I", v[2]).await
    }
}
