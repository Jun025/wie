use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::rms::RecordStore;

const ABSTRACT: MethodAccessFlags = MethodAccessFlags::PUBLIC.union(MethodAccessFlags::ABSTRACT);

// interface javax.microedition.rms.RecordEnumeration
pub struct RecordEnumeration;

impl RecordEnumeration {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/rms/RecordEnumeration",
            parent_class: None,
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new_abstract("numRecords", "()I", ABSTRACT),
                JavaMethodProto::new_abstract("nextRecord", "()[B", ABSTRACT),
                JavaMethodProto::new_abstract("nextRecordId", "()I", ABSTRACT),
                JavaMethodProto::new_abstract("previousRecord", "()[B", ABSTRACT),
                JavaMethodProto::new_abstract("previousRecordId", "()I", ABSTRACT),
                JavaMethodProto::new_abstract("hasNextElement", "()Z", ABSTRACT),
                JavaMethodProto::new_abstract("hasPreviousElement", "()Z", ABSTRACT),
                JavaMethodProto::new_abstract("reset", "()V", ABSTRACT),
                JavaMethodProto::new_abstract("rebuild", "()V", ABSTRACT),
                JavaMethodProto::new_abstract("keepUpdated", "(Z)V", ABSTRACT),
                JavaMethodProto::new_abstract("isKeptUpdated", "()Z", ABSTRACT),
                JavaMethodProto::new_abstract("destroy", "()V", ABSTRACT),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::INTERFACE | ClassAccessFlags::ABSTRACT,
        }
    }
}

// class net.wie.RecordEnumerationImpl — what RecordStore.enumerateRecords returns.
//
// The id list is taken once, already filtered and ordered, when the enumeration is made.
// ponytail: keepUpdated is accepted and ignored and rebuild is a no-op — a store changed while an
// enumeration is open is not seen by it. Re-run the filter/sort in rebuild if a game relies on that.
pub struct RecordEnumerationImpl;

impl RecordEnumerationImpl {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/RecordEnumerationImpl",
            parent_class: Some("java/lang/Object"),
            interfaces: vec!["javax/microedition/rms/RecordEnumeration"],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Ljavax/microedition/rms/RecordStore;[I)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("numRecords", "()I", Self::num_records, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("nextRecord", "()[B", Self::next_record, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("nextRecordId", "()I", Self::next_record_id, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("previousRecord", "()[B", Self::previous_record, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("previousRecordId", "()I", Self::previous_record_id, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hasNextElement", "()Z", Self::has_next_element, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hasPreviousElement", "()Z", Self::has_previous_element, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("reset", "()V", Self::reset, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("rebuild", "()V", Self::rebuild, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keepUpdated", "(Z)V", Self::keep_updated, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("isKeptUpdated", "()Z", Self::is_kept_updated, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("destroy", "()V", Self::destroy, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("store", "Ljavax/microedition/rms/RecordStore;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("ids", "[I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("index", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        store: ClassInstanceRef<RecordStore>,
        ids: ClassInstanceRef<Array<i32>>,
    ) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "store", "Ljavax/microedition/rms/RecordStore;", store).await?;
        jvm.put_field(&mut this, "ids", "[I", ids).await?;
        jvm.put_field(&mut this, "index", "I", -1).await?;

        Ok(())
    }

    async fn num_records(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let ids: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, "ids", "[I").await?;

        Ok(jvm.array_length(&ids).await? as _)
    }

    async fn has_next_element(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        let index: i32 = jvm.get_field(&this, "index", "I").await?;

        Ok(index != Self::num_records(jvm, context, this).await? - 1)
    }

    async fn has_previous_element(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        let index: i32 = jvm.get_field(&this, "index", "I").await?;

        Ok(Self::num_records(jvm, context, this).await? > 0 && index != 0)
    }

    // MIDP 2.0 (JSR118) current-position model, as in the RI: `index` is the element last returned, -1 right
    // after creation or reset(). From -1, next gives the first element and previous gives the LAST; otherwise
    // each call moves one step, so next-then-previous gives the element before, not the same one again.
    async fn step(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, forward: bool) -> JvmResult<i32> {
        let index: i32 = jvm.get_field(&this, "index", "I").await?;
        let count = Self::num_records(jvm, context, this.clone()).await?;
        let at = match (index, forward) {
            (-1, true) => 0,
            (-1, false) => count - 1,
            (_, true) => index + 1,
            (_, false) => index - 1,
        };
        if at < 0 || at >= count {
            return Err(jvm.exception("javax/microedition/rms/InvalidRecordIDException", "No more records").await);
        }

        let ids: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, "ids", "[I").await?;
        let id: i32 = jvm.load_array(&ids, at as _, 1).await?[0];
        jvm.put_field(&mut this, "index", "I", at).await?;

        Ok(id)
    }

    async fn record(jvm: &Jvm, this: &ClassInstanceRef<Self>, id: i32) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        let store: ClassInstanceRef<RecordStore> = jvm.get_field(this, "store", "Ljavax/microedition/rms/RecordStore;").await?;

        jvm.invoke_virtual(&store, "javax/microedition/rms/RecordStore", "getRecord", "(I)[B", (id,))
            .await
    }

    async fn next_record_id(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        Self::step(jvm, context, this, true).await
    }

    async fn previous_record_id(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        Self::step(jvm, context, this, false).await
    }

    async fn next_record(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        let id = Self::step(jvm, context, this.clone(), true).await?;

        Self::record(jvm, &this, id).await
    }

    async fn previous_record(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        let id = Self::step(jvm, context, this.clone(), false).await?;

        Self::record(jvm, &this, id).await
    }

    async fn reset(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        jvm.put_field(&mut this, "index", "I", -1).await
    }

    async fn rebuild(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub net.wie.RecordEnumerationImpl::rebuild({this:?})");

        Ok(())
    }

    async fn keep_updated(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, keep: bool) -> JvmResult<()> {
        tracing::warn!("stub net.wie.RecordEnumerationImpl::keepUpdated({this:?}, {keep})");

        Ok(())
    }

    async fn is_kept_updated(_: &Jvm, _: &mut WieJvmContext, _: ClassInstanceRef<Self>) -> JvmResult<bool> {
        Ok(false)
    }

    async fn destroy(_: &Jvm, _: &mut WieJvmContext, _: ClassInstanceRef<Self>) -> JvmResult<()> {
        Ok(())
    }
}
