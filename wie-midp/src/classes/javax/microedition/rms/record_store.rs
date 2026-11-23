use alloc::{borrow::ToOwned, boxed::Box, vec, vec::Vec};

use bytemuck::cast_vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_backend::Database;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::rms::{RecordComparator, RecordEnumeration, RecordFilter};

// class javax.microedition.rms.RecordStore
pub struct RecordStore;

impl RecordStore {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/rms/RecordStore",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, MethodAccessFlags::PRIVATE),
                JavaMethodProto::new("addRecord", "([BII)I", Self::add_record, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("deleteRecord", "(I)V", Self::delete_record, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getSizeAvailable", "()I", Self::get_size_available, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getNextRecordID", "()I", Self::get_next_record_id, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getRecord", "(I)[B", Self::get_record, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getRecord", "(I[BI)I", Self::get_record_array, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getRecordSize", "(I)I", Self::get_record_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setRecord", "(I[BII)V", Self::set_record, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getNumRecords", "()I", Self::get_num_records, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "enumerateRecords",
                    "(Ljavax/microedition/rms/RecordFilter;Ljavax/microedition/rms/RecordComparator;Z)Ljavax/microedition/rms/RecordEnumeration;",
                    Self::enumerate_records,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("closeRecordStore", "()V", Self::close_record_store, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "openRecordStore",
                    "(Ljava/lang/String;Z)Ljavax/microedition/rms/RecordStore;",
                    Self::open_record_store,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "deleteRecordStore",
                    "(Ljava/lang/String;)V",
                    Self::delete_record_store,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "listRecordStores",
                    "()[Ljava/lang/String;",
                    Self::list_record_stores,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![JavaFieldProto::new("dbName", "Ljava/lang/String;", FieldAccessFlags::PRIVATE)],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, db_name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.rms.RecordStore::<init>({this:?}, {db_name:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "javax/microedition/rms/RecordStore", "dbName", "Ljava/lang/String;", db_name)
            .await?;

        Ok(())
    }

    async fn add_record(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.rms.RecordStore::addRecord({this:?}, {data:?}, {offset}, {length})");

        let mut database = Self::get_database(jvm, context, &this).await?;

        let data: Vec<i8> = if data.is_null() {
            if length != 0 {
                return Err(jvm.exception("java/lang/NullPointerException", "Record data is null").await);
            }
            Vec::new()
        } else {
            jvm.load_array(&data, offset as _, length as _).await?
        };

        let id = database.add(&cast_vec(data)).await;

        Ok(id as _)
    }

    async fn delete_record(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, record_id: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.rms.RecordStore::deleteRecord({this:?}, {record_id})");

        let mut database = Self::get_database(jvm, context, &this).await?;
        if !database.delete(record_id as _).await {
            return Err(jvm.exception("javax/microedition/rms/InvalidRecordIDException", "Record not found").await);
        }

        Ok(())
    }

    async fn get_size_available(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.rms.RecordStore::getSizeAvailable({this:?})");

        Ok(1000000 as _)
    }

    async fn get_next_record_id(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.rms.RecordStore::getNextRecordID({this:?})");

        let database = Self::get_database(jvm, context, &this).await?;

        let next_id = database.next_id().await;

        Ok(next_id as _)
    }

    async fn get_record(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        record_id: i32,
    ) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        tracing::debug!("javax.microedition.rms.RecordStore::getRecord({this:?}, {record_id})");

        let database = Self::get_database(jvm, context, &this).await?;

        let result = database.get(record_id as _).await;
        if result.is_none() {
            return Err(jvm.exception("javax/microedition/rms/InvalidRecordIDException", "Record not found").await);
        }

        // fork: upstream returns null for a record with no data (MIDP). Ours returns the empty array,
        // as before the 2026-11-23 merge.
        let data = result.unwrap();

        let mut array = jvm.instantiate_array("B", data.len() as _).await?;
        jvm.store_array(&mut array, 0, cast_vec::<u8, i8>(data)).await?;

        Ok(array.into())
    }

    async fn get_record_array(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        record_id: i32,
        mut buffer: ClassInstanceRef<Array<i8>>,
        offset: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.rms.RecordStore::getRecord({this:?}, {record_id}, {buffer:?}, {offset})");

        let database = Self::get_database(jvm, context, &this).await?;

        let result = database.get(record_id as _).await;
        if result.is_none() {
            return Err(jvm.exception("javax/microedition/rms/InvalidRecordIDException", "Record not found").await);
        }

        let data = result.unwrap();
        let data_length = data.len();
        jvm.store_array(&mut buffer, offset as _, cast_vec::<u8, i8>(data)).await?;

        Ok(data_length as _)
    }

    async fn get_record_size(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, record_id: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.rms.RecordStore::getRecordSize({this:?}, {record_id})");

        let database = Self::get_database(jvm, context, &this).await?;

        let result = database.get(record_id as _).await;
        if result.is_none() {
            return Err(jvm.exception("javax/microedition/rms/InvalidRecordIDException", "Record not found").await);
        }

        let data = result.unwrap();

        Ok(data.len() as _)
    }

    async fn set_record(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        record_id: i32,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.rms.RecordStore::setRecord({this:?}, {record_id}, {data:?}, {offset}, {length})");

        let data: Vec<i8> = if data.is_null() {
            if length != 0 {
                return Err(jvm.exception("java/lang/NullPointerException", "Record data is null").await);
            }
            Vec::new()
        } else {
            jvm.load_array(&data, offset as _, length as _).await?
        };

        let mut database = Self::get_database(jvm, context, &this).await?;
        database.set(record_id as _, &cast_vec(data)).await;

        Ok(())
    }

    async fn get_num_records(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.rms.RecordStore::getNumRecords({this:?})");

        let database = Self::get_database(jvm, context, &this).await?;

        let count = database.get_record_ids().await.len();

        Ok(count as _)
    }

    async fn enumerate_records(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        filter: ClassInstanceRef<RecordFilter>,
        comparator: ClassInstanceRef<RecordComparator>,
        keep_updated: bool,
    ) -> JvmResult<ClassInstanceRef<RecordEnumeration>> {
        tracing::debug!("javax.microedition.rms.RecordStore::enumerateRecords({this:?}, {filter:?}, {comparator:?}, {keep_updated})");
        Ok(jvm
            .new_class(
                "net/wie/RecordEnumeration",
                "(Ljavax/microedition/rms/RecordStore;Ljavax/microedition/rms/RecordFilter;Ljavax/microedition/rms/RecordComparator;Z)V",
                (this, filter, comparator, keep_updated),
            )
            .await?
            .into())
    }

    async fn close_record_store(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.rms.RecordStore::closeRecordStore({this:?})");

        Ok(())
    }

    async fn open_record_store(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        name: ClassInstanceRef<String>,
        create: bool,
    ) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("javax.microedition.rms.RecordStore::openRecordStore({name:?}, {create:?})");

        // MIDP: without createIfNecessary a missing store throws. Titles detect their first run by
        // catching this; opening an empty store instead left them reading record 1 into null.
        let db_name = JavaLangString::to_rust_string(jvm, &name).await?;
        let system = context.system();
        let pid = system.pid().to_owned();
        let repository = system.platform().database_repository();
        if create {
            repository.create(&db_name, &pid).await;
        } else if !repository.exists(&db_name, &pid).await {
            return Err(jvm.exception("javax/microedition/rms/RecordStoreNotFoundException", &db_name).await);
        }

        let store = jvm
            .new_class("javax/microedition/rms/RecordStore", "(Ljava/lang/String;)V", (name,))
            .await?;

        Ok(store.into())
    }

    async fn delete_record_store(_jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.rms.RecordStore::deleteRecordStore({name:?})");

        Ok(())
    }

    async fn list_record_stores(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Array<String>>> {
        tracing::warn!("stub javax.microedition.rms.RecordStore::listRecordStores()");

        let result = jvm.instantiate_array("Ljava/lang/String;", 0).await?;

        Ok(result.into())
    }

    pub async fn get_database(jvm: &Jvm, context: &mut WieJvmContext, this: &ClassInstanceRef<Self>) -> JvmResult<Box<dyn Database>> {
        let db_name = jvm
            .get_field(this, "javax/microedition/rms/RecordStore", "dbName", "Ljava/lang/String;")
            .await?;
        let db_name_str = JavaLangString::to_rust_string(jvm, &db_name).await?;

        let system = context.system();
        let pid = system.pid().to_owned();

        Ok(system.platform().database_repository().open(&db_name_str, &pid).await)
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec, vec::Vec};

    use jvm::{Array, ClassInstanceRef, JavaError, Jvm, Result as JvmResult, runtime::JavaLangString};
    use jvm_class_proto::JavaMethodProto;
    use jvm_types::{ClassAccessFlags, MethodAccessFlags};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_util::Result;

    use crate::get_protos;

    use super::{RecordComparator, RecordEnumeration, RecordFilter, RecordStore};

    #[test]
    fn empty_records_can_be_updated_and_deleted() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "delete-record").await?.into();
            let store: ClassInstanceRef<RecordStore> = jvm
                .invoke_static(
                    "javax/microedition/rms/RecordStore",
                    "openRecordStore",
                    "(Ljava/lang/String;Z)Ljavax/microedition/rms/RecordStore;",
                    (name, true),
                )
                .await?;

            let record_id: i32 = jvm
                .invoke_virtual(
                    &store,
                    "javax/microedition/rms/RecordStore",
                    "addRecord",
                    "([BII)I",
                    (ClassInstanceRef::<Array<i8>>::from(None), 0, 0),
                )
                .await?;
            assert_eq!(record_id, 1);
            let empty: ClassInstanceRef<Array<i8>> = jvm
                .invoke_virtual(&store, "javax/microedition/rms/RecordStore", "getRecord", "(I)[B", (record_id,))
                .await?;
            assert_eq!(jvm.array_length(&empty).await?, 0); // fork: an empty array, not null

            let mut data = jvm.instantiate_array("B", 2).await?;
            jvm.store_array(&mut data, 0, [1i8, 2]).await?;
            let _: () = jvm
                .invoke_virtual(
                    &store,
                    "javax/microedition/rms/RecordStore",
                    "setRecord",
                    "(I[BII)V",
                    (record_id, data, 0, 2),
                )
                .await?;
            let loaded: ClassInstanceRef<Array<i8>> = jvm
                .invoke_virtual(&store, "javax/microedition/rms/RecordStore", "getRecord", "(I)[B", (record_id,))
                .await?;
            assert_eq!(jvm.load_array::<i8>(&loaded, 0, 2).await?, [1, 2]);

            let _: () = jvm
                .invoke_virtual(
                    &store,
                    "javax/microedition/rms/RecordStore",
                    "setRecord",
                    "(I[BII)V",
                    (record_id, ClassInstanceRef::<Array<i8>>::from(None), 0, 0),
                )
                .await?;
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&store, "javax/microedition/rms/RecordStore", "getRecordSize", "(I)I", (record_id,))
                    .await?,
                0
            );

            let count: i32 = jvm
                .invoke_virtual(&store, "javax/microedition/rms/RecordStore", "getNumRecords", "()I", ())
                .await?;
            assert_eq!(count, 1);

            let snapshot: ClassInstanceRef<RecordEnumeration> = jvm
                .invoke_virtual(
                    &store,
                    "javax/microedition/rms/RecordStore",
                    "enumerateRecords",
                    "(Ljavax/microedition/rms/RecordFilter;Ljavax/microedition/rms/RecordComparator;Z)Ljavax/microedition/rms/RecordEnumeration;",
                    (
                        ClassInstanceRef::<RecordFilter>::from(None),
                        ClassInstanceRef::<RecordComparator>::from(None),
                        false,
                    ),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(&store, "javax/microedition/rms/RecordStore", "deleteRecord", "(I)V", (record_id,))
                .await?;
            let count: i32 = jvm
                .invoke_virtual(&store, "javax/microedition/rms/RecordStore", "getNumRecords", "()I", ())
                .await?;
            assert_eq!(count, 0);

            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&snapshot, "javax/microedition/rms/RecordEnumeration", "nextRecordId", "()I", ())
                    .await?,
                record_id
            );
            let _: () = jvm
                .invoke_virtual(&snapshot, "javax/microedition/rms/RecordEnumeration", "reset", "()V", ())
                .await?;
            let stale: JvmResult<ClassInstanceRef<Array<i8>>> = jvm
                .invoke_virtual(&snapshot, "javax/microedition/rms/RecordEnumeration", "nextRecord", "()[B", ())
                .await;
            let Err(JavaError::JavaException(exception)) = stale else {
                panic!("deleted snapshot record lookup succeeded")
            };
            assert!(jvm.is_instance(&*exception, "javax/microedition/rms/InvalidRecordIDException"));

            let deleted: JvmResult<ClassInstanceRef<Array<i8>>> = jvm
                .invoke_virtual(&store, "javax/microedition/rms/RecordStore", "getRecord", "(I)[B", (record_id,))
                .await;
            let Err(JavaError::JavaException(exception)) = deleted else {
                panic!("deleted record lookup succeeded");
            };
            assert!(jvm.is_instance(&*exception, "javax/microedition/rms/InvalidRecordIDException"));

            let unknown: JvmResult<()> = jvm
                .invoke_virtual(&store, "javax/microedition/rms/RecordStore", "deleteRecord", "(I)V", (99,))
                .await;
            let Err(JavaError::JavaException(exception)) = unknown else {
                panic!("unknown record deletion succeeded");
            };
            assert!(jvm.is_instance(&*exception, "javax/microedition/rms/InvalidRecordIDException"));
            let _: () = jvm
                .invoke_virtual(&snapshot, "javax/microedition/rms/RecordEnumeration", "rebuild", "()V", ())
                .await?;
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&snapshot, "javax/microedition/rms/RecordEnumeration", "numRecords", "()I", ())
                    .await?,
                0
            );
            for method in ["hasNextElement", "hasPreviousElement"] {
                assert!(
                    !jvm.invoke_virtual::<_, bool>(&snapshot, "javax/microedition/rms/RecordEnumeration", method, "()Z", ())
                        .await?
                );
            }

            Ok(())
        })
    }

    struct PositiveRecordOrder;

    impl PositiveRecordOrder {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/PositiveRecordOrder",
                parent_class: Some("java/lang/Object"),
                interfaces: vec!["javax/microedition/rms/RecordFilter", "javax/microedition/rms/RecordComparator"],
                methods: vec![
                    JavaMethodProto::new("matches", "([B)Z", Self::matches, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("compare", "([B[B)I", Self::compare, MethodAccessFlags::PUBLIC),
                ],
                fields: vec![],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn matches(jvm: &Jvm, _: &mut WieJvmContext, _: ClassInstanceRef<Self>, data: ClassInstanceRef<Array<i8>>) -> JvmResult<bool> {
            Ok(jvm.load_array::<i8>(&data, 0, 1).await?[0] > 0)
        }

        async fn compare(
            jvm: &Jvm,
            _: &mut WieJvmContext,
            _: ClassInstanceRef<Self>,
            left: ClassInstanceRef<Array<i8>>,
            right: ClassInstanceRef<Array<i8>>,
        ) -> JvmResult<i32> {
            let left = jvm.load_array::<i8>(&left, 0, 1).await?[0];
            let right = jvm.load_array::<i8>(&right, 0, 1).await?[0];
            Ok(i32::from(left).cmp(&i32::from(right)) as i32)
        }
    }

    #[test]
    fn record_enumeration_filters_sorts_and_tracks_changes() -> Result<()> {
        run_jvm_test(
            Box::new([get_protos().into(), Box::new([PositiveRecordOrder::as_proto()])]),
            |jvm| async move {
                let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "enumeration").await?.into();
                let missing: JvmResult<ClassInstanceRef<RecordStore>> = jvm
                    .invoke_static(
                        "javax/microedition/rms/RecordStore",
                        "openRecordStore",
                        "(Ljava/lang/String;Z)Ljavax/microedition/rms/RecordStore;",
                        (name.clone(), false),
                    )
                    .await;
                let Err(JavaError::JavaException(exception)) = missing else {
                    panic!("opening an absent record store without creation succeeded");
                };
                assert!(jvm.is_instance(&*exception, "javax/microedition/rms/RecordStoreNotFoundException"));
                let store: ClassInstanceRef<RecordStore> = jvm
                    .invoke_static(
                        "javax/microedition/rms/RecordStore",
                        "openRecordStore",
                        "(Ljava/lang/String;Z)Ljavax/microedition/rms/RecordStore;",
                        (name.clone(), true),
                    )
                    .await?;
                let other: ClassInstanceRef<RecordStore> = jvm
                    .invoke_static(
                        "javax/microedition/rms/RecordStore",
                        "openRecordStore",
                        "(Ljava/lang/String;Z)Ljavax/microedition/rms/RecordStore;",
                        (name, false),
                    )
                    .await?;
                let order = jvm.instantiate_class("test/PositiveRecordOrder").await?;
                let filter: ClassInstanceRef<RecordFilter> = order.clone().into();
                let comparator: ClassInstanceRef<RecordComparator> = order.into();
                let mut data = jvm.instantiate_array("B", 1).await?;
                for value in [30i8, 10, 20] {
                    jvm.store_array(&mut data, 0, [value]).await?;
                    let _: i32 = jvm
                        .invoke_virtual(&store, "javax/microedition/rms/RecordStore", "addRecord", "([BII)I", (data.clone(), 0, 1))
                        .await?;
                }
                let mut enumerations = vec![];
                for keep_updated in [false, true, true] {
                    let enumeration: ClassInstanceRef<RecordEnumeration> = jvm.invoke_virtual(
                    &store, "javax/microedition/rms/RecordStore", "enumerateRecords",
                    "(Ljavax/microedition/rms/RecordFilter;Ljavax/microedition/rms/RecordComparator;Z)Ljavax/microedition/rms/RecordEnumeration;",
                    (filter.clone(), comparator.clone(), keep_updated),
                ).await?;
                    assert!(jvm.is_instance(&**enumeration, "javax/microedition/rms/RecordEnumeration"));
                    assert_eq!(
                        jvm.invoke_virtual::<_, i32>(&enumeration, "javax/microedition/rms/RecordEnumeration", "numRecords", "()I", ())
                            .await?,
                        3
                    );
                    enumerations.push(enumeration);
                }
                let snapshot = &enumerations[0];
                let forward = &enumerations[1];
                let backward = &enumerations[2];
                for expected in [2, 3, 1] {
                    assert_eq!(
                        jvm.invoke_virtual::<_, i32>(snapshot, "javax/microedition/rms/RecordEnumeration", "nextRecordId", "()I", ())
                            .await?,
                        expected
                    );
                }
                assert!(
                    !jvm.invoke_virtual::<_, bool>(snapshot, "javax/microedition/rms/RecordEnumeration", "hasNextElement", "()Z", ())
                        .await?
                );
                for _ in 0..2 {
                    let end: JvmResult<i32> = jvm
                        .invoke_virtual(snapshot, "javax/microedition/rms/RecordEnumeration", "nextRecordId", "()I", ())
                        .await;
                    let Err(JavaError::JavaException(exception)) = end else {
                        panic!("enumeration wrapped at end")
                    };
                    assert!(jvm.is_instance(&*exception, "javax/microedition/rms/InvalidRecordIDException"));
                }
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(snapshot, "javax/microedition/rms/RecordEnumeration", "previousRecordId", "()I", ())
                        .await?,
                    3
                );
                let _: () = jvm
                    .invoke_virtual(snapshot, "javax/microedition/rms/RecordEnumeration", "reset", "()V", ())
                    .await?;
                for expected in [1, 3, 2] {
                    assert_eq!(
                        jvm.invoke_virtual::<_, i32>(snapshot, "javax/microedition/rms/RecordEnumeration", "previousRecordId", "()I", ())
                            .await?,
                        expected
                    );
                }
                assert!(
                    !jvm.invoke_virtual::<_, bool>(snapshot, "javax/microedition/rms/RecordEnumeration", "hasPreviousElement", "()Z", ())
                        .await?
                );
                for _ in 0..2 {
                    let start: JvmResult<i32> = jvm
                        .invoke_virtual(snapshot, "javax/microedition/rms/RecordEnumeration", "previousRecordId", "()I", ())
                        .await;
                    let Err(JavaError::JavaException(exception)) = start else {
                        panic!("enumeration wrapped at start")
                    };
                    assert!(jvm.is_instance(&*exception, "javax/microedition/rms/InvalidRecordIDException"));
                }
                for expected in [2, 3] {
                    assert_eq!(
                        jvm.invoke_virtual::<_, i32>(forward, "javax/microedition/rms/RecordEnumeration", "nextRecordId", "()I", ())
                            .await?,
                        expected
                    );
                }
                for expected in [1, 3] {
                    assert_eq!(
                        jvm.invoke_virtual::<_, i32>(backward, "javax/microedition/rms/RecordEnumeration", "previousRecordId", "()I", ())
                            .await?,
                        expected
                    );
                }
                let _: () = jvm
                    .invoke_virtual(&other, "javax/microedition/rms/RecordStore", "deleteRecord", "(I)V", (3,))
                    .await?;
                assert!(
                    jvm.invoke_virtual::<_, bool>(forward, "javax/microedition/rms/RecordEnumeration", "hasNextElement", "()Z", ())
                        .await?
                );
                assert!(
                    jvm.invoke_virtual::<_, bool>(backward, "javax/microedition/rms/RecordEnumeration", "hasPreviousElement", "()Z", ())
                        .await?
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(forward, "javax/microedition/rms/RecordEnumeration", "nextRecordId", "()I", ())
                        .await?,
                    1
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(backward, "javax/microedition/rms/RecordEnumeration", "previousRecordId", "()I", ())
                        .await?,
                    2
                );

                jvm.store_array(&mut data, 0, [15i8]).await?;
                let _: () = jvm
                    .invoke_virtual(
                        &other,
                        "javax/microedition/rms/RecordStore",
                        "setRecord",
                        "(I[BII)V",
                        (2, data.clone(), 0, 1),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(snapshot, "javax/microedition/rms/RecordEnumeration", "reset", "()V", ())
                    .await?;
                let mut bytes: ClassInstanceRef<Array<i8>> = jvm
                    .invoke_virtual(snapshot, "javax/microedition/rms/RecordEnumeration", "nextRecord", "()[B", ())
                    .await?;
                assert_eq!(jvm.load_array::<i8>(&bytes, 0, 1).await?, [15]);
                jvm.store_array(&mut bytes, 0, [99i8]).await?;
                let stored: ClassInstanceRef<Array<i8>> = jvm
                    .invoke_virtual(&other, "javax/microedition/rms/RecordStore", "getRecord", "(I)[B", (2,))
                    .await?;
                assert_eq!(jvm.load_array::<i8>(&stored, 0, 1).await?, [15]);

                jvm.store_array(&mut data, 0, [25i8]).await?;
                let fourth: i32 = jvm
                    .invoke_virtual(&other, "javax/microedition/rms/RecordStore", "addRecord", "([BII)I", (data.clone(), 0, 1))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(forward, "javax/microedition/rms/RecordEnumeration", "keepUpdated", "(Z)V", (false,))
                    .await?;
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(forward, "javax/microedition/rms/RecordEnumeration", "numRecords", "()I", ())
                        .await?,
                    3
                );
                jvm.store_array(&mut data, 0, [5i8]).await?;
                let fifth: i32 = jvm
                    .invoke_virtual(&other, "javax/microedition/rms/RecordStore", "addRecord", "([BII)I", (data.clone(), 0, 1))
                    .await?;
                assert!(
                    !jvm.invoke_virtual::<_, bool>(forward, "javax/microedition/rms/RecordEnumeration", "isKeptUpdated", "()Z", ())
                        .await?
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(forward, "javax/microedition/rms/RecordEnumeration", "numRecords", "()I", ())
                        .await?,
                    3
                );
                let _: () = jvm
                    .invoke_virtual(forward, "javax/microedition/rms/RecordEnumeration", "keepUpdated", "(Z)V", (true,))
                    .await?;
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(forward, "javax/microedition/rms/RecordEnumeration", "nextRecordId", "()I", ())
                        .await?,
                    fifth
                );
                jvm.store_array(&mut data, 0, [-1i8]).await?;
                let _: () = jvm
                    .invoke_virtual(
                        &other,
                        "javax/microedition/rms/RecordStore",
                        "setRecord",
                        "(I[BII)V",
                        (fifth, data.clone(), 0, 1),
                    )
                    .await?;
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(forward, "javax/microedition/rms/RecordEnumeration", "numRecords", "()I", ())
                        .await?,
                    3
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(forward, "javax/microedition/rms/RecordEnumeration", "nextRecordId", "()I", ())
                        .await?,
                    2
                );
                jvm.store_array(&mut data, 0, [1i8]).await?;
                let _: () = jvm
                    .invoke_virtual(
                        &other,
                        "javax/microedition/rms/RecordStore",
                        "setRecord",
                        "(I[BII)V",
                        (1, data.clone(), 0, 1),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(snapshot, "javax/microedition/rms/RecordEnumeration", "rebuild", "()V", ())
                    .await?;
                for expected in [1, 2, fourth] {
                    assert_eq!(
                        jvm.invoke_virtual::<_, i32>(snapshot, "javax/microedition/rms/RecordEnumeration", "nextRecordId", "()I", ())
                            .await?,
                        expected
                    );
                }
                let _: () = jvm
                    .invoke_virtual(snapshot, "javax/microedition/rms/RecordEnumeration", "reset", "()V", ())
                    .await?;
                let bytes: ClassInstanceRef<Array<i8>> = jvm
                    .invoke_virtual(snapshot, "javax/microedition/rms/RecordEnumeration", "previousRecord", "()[B", ())
                    .await?;
                assert_eq!(jvm.load_array::<i8>(&bytes, 0, 1).await?, [25]);
                let _: () = jvm
                    .invoke_virtual(snapshot, "javax/microedition/rms/RecordEnumeration", "destroy", "()V", ())
                    .await?;
                for method in ["hasNextElement", "hasPreviousElement", "isKeptUpdated"] {
                    let destroyed: JvmResult<bool> = jvm
                        .invoke_virtual(snapshot, "javax/microedition/rms/RecordEnumeration", method, "()Z", ())
                        .await;
                    let Err(JavaError::JavaException(exception)) = destroyed else {
                        panic!("destroyed enumeration remained usable")
                    };
                    assert!(jvm.is_instance(&*exception, "java/lang/IllegalStateException"));
                }
                Ok(())
            },
        )
    }
    // A title's first run is `openRecordStore(name, false)` throwing; after a create it opens.
    #[test]
    fn open_without_create_throws_until_the_store_exists() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let open = async |create: bool| -> JvmResult<ClassInstanceRef<RecordStore>> {
                let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "first-run").await?.into();
                jvm.invoke_static(
                    "javax/microedition/rms/RecordStore",
                    "openRecordStore",
                    "(Ljava/lang/String;Z)Ljavax/microedition/rms/RecordStore;",
                    (name, create),
                )
                .await
            };

            let Err(JavaError::JavaException(exception)) = open(false).await else {
                panic!("a missing store opened without createIfNecessary");
            };
            assert!(jvm.is_instance(&*exception, "javax/microedition/rms/RecordStoreNotFoundException"));

            open(true).await?;
            open(false).await?;

            Ok(())
        })
    }

    // Reverse order by first byte, and a filter that drops records whose first byte is 2.
    struct ReverseFirstByte;

    impl ReverseFirstByte {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "TestReverseFirstByte",
                parent_class: Some("java/lang/Object"),
                interfaces: vec!["javax/microedition/rms/RecordComparator", "javax/microedition/rms/RecordFilter"],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("compare", "([B[B)I", Self::compare, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("matches", "([B)Z", Self::matches, MethodAccessFlags::PUBLIC),
                ],
                fields: vec![],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
        }

        async fn compare(
            jvm: &Jvm,
            _: &mut WieJvmContext,
            _: ClassInstanceRef<Self>,
            a: ClassInstanceRef<Array<i8>>,
            b: ClassInstanceRef<Array<i8>>,
        ) -> JvmResult<i32> {
            let a: Vec<i8> = jvm.load_array(&a, 0, 1).await?;
            let b: Vec<i8> = jvm.load_array(&b, 0, 1).await?;

            Ok((b[0] as i32 - a[0] as i32).signum())
        }

        async fn matches(jvm: &Jvm, _: &mut WieJvmContext, _: ClassInstanceRef<Self>, a: ClassInstanceRef<Array<i8>>) -> JvmResult<bool> {
            let a: Vec<i8> = jvm.load_array(&a, 0, 1).await?;

            Ok(a[0] != 2)
        }
    }

    /// 66959afab216 (SKT) died at boot on `NoSuchMethodError: RecordStore.enumerateRecords(RecordFilter,
    /// RecordComparator,Z)`. Walks the enumeration both ways with no filter/comparator (ascending ids),
    /// then with a guest filter and comparator, which is the part that calls back into Java.
    #[test]
    fn enumerate_records_filters_orders_and_walks_both_ways() -> Result<()> {
        run_jvm_test(
            Box::new([get_protos().into(), Box::new([ReverseFirstByte::as_proto()])]),
            |jvm| async move {
                let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "enumerate").await?.into();
                let store: ClassInstanceRef<RecordStore> = jvm
                    .invoke_static(
                        "javax/microedition/rms/RecordStore",
                        "openRecordStore",
                        "(Ljava/lang/String;Z)Ljavax/microedition/rms/RecordStore;",
                        (name, true),
                    )
                    .await?;
                for first in [1i8, 3, 2] {
                    let mut data = jvm.instantiate_array("B", 1).await?;
                    jvm.store_array(&mut data, 0, [first]).await?;
                    let _: i32 = jvm
                        .invoke_virtual(&store, "javax/microedition/rms/RecordStore", "addRecord", "([BII)I", (data, 0, 1))
                        .await?;
                }

                const ENUMERATE: &str =
                    "(Ljavax/microedition/rms/RecordFilter;Ljavax/microedition/rms/RecordComparator;Z)Ljavax/microedition/rms/RecordEnumeration;";
                const E: &str = "javax/microedition/rms/RecordEnumeration";
                let no_filter: ClassInstanceRef<RecordFilter> = None.into();
                let no_comparator: ClassInstanceRef<RecordComparator> = None.into();

                let plain: ClassInstanceRef<RecordEnumeration> = jvm
                    .invoke_virtual(
                        &store,
                        "javax/microedition/rms/RecordStore",
                        "enumerateRecords",
                        ENUMERATE,
                        (no_filter, no_comparator, false),
                    )
                    .await?;
                let count: i32 = jvm.invoke_virtual(&plain, E, "numRecords", "()I", ()).await?;
                assert_eq!(count, 3);
                // JSR118: right after creation previousRecord() returns the LAST element, so there is one.
                let fresh_has_previous: bool = jvm.invoke_virtual(&plain, E, "hasPreviousElement", "()Z", ()).await?;
                assert!(fresh_has_previous);
                let mut ids = Vec::new();
                while jvm.invoke_virtual::<_, bool>(&plain, E, "hasNextElement", "()Z", ()).await? {
                    ids.push(jvm.invoke_virtual::<_, i32>(&plain, E, "nextRecordId", "()I", ()).await?);
                }
                assert_eq!(ids, [1, 2, 3]);
                // Current-position model: from the last element (3), previous is the one before it.
                let back: i32 = jvm.invoke_virtual(&plain, E, "previousRecordId", "()I", ()).await?;
                assert_eq!(back, 2);

                // reset() = state right after creation, so walking backwards starts at the last element.
                let _: () = jvm.invoke_virtual(&plain, E, "reset", "()V", ()).await?;
                let mut ids = Vec::new();
                while jvm.invoke_virtual::<_, bool>(&plain, E, "hasPreviousElement", "()Z", ()).await? {
                    ids.push(jvm.invoke_virtual::<_, i32>(&plain, E, "previousRecordId", "()I", ()).await?);
                }
                assert_eq!(ids, [3, 2, 1]);
                let past_start: JvmResult<i32> = jvm.invoke_virtual(&plain, E, "previousRecordId", "()I", ()).await;
                assert!(matches!(past_start, Err(JavaError::JavaException(_))));

                let ordering = jvm.new_class("TestReverseFirstByte", "()V", ()).await?;
                let sorted: ClassInstanceRef<RecordEnumeration> = jvm
                    .invoke_virtual(
                        &store,
                        "javax/microedition/rms/RecordStore",
                        "enumerateRecords",
                        ENUMERATE,
                        (ordering.clone(), ordering, false),
                    )
                    .await?;
                let mut firsts = Vec::new();
                while jvm.invoke_virtual::<_, bool>(&sorted, E, "hasNextElement", "()Z", ()).await? {
                    let record: ClassInstanceRef<Array<i8>> = jvm.invoke_virtual(&sorted, E, "nextRecord", "()[B", ()).await?;
                    let record: Vec<i8> = jvm.load_array(&record, 0, 1).await?;
                    firsts.push(record[0]);
                }
                assert_eq!(firsts, [3i8, 1]);

                Ok(())
            },
        )
    }
}
