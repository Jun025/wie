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
                JavaMethodProto::new("closeRecordStore", "()V", Self::close_record_store, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "enumerateRecords",
                    "(Ljavax/microedition/rms/RecordFilter;Ljavax/microedition/rms/RecordComparator;Z)Ljavax/microedition/rms/RecordEnumeration;",
                    Self::enumerate_records,
                    MethodAccessFlags::PUBLIC,
                ),
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

        jvm.put_field(&mut this, "dbName", "Ljava/lang/String;", db_name).await?;

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

        let data: Vec<i8> = jvm.load_array(&data, offset as _, length as _).await?;

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

        let data: Vec<i8> = jvm.load_array(&data, offset as _, length as _).await?;

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

    async fn close_record_store(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.rms.RecordStore::closeRecordStore({this:?})");

        Ok(())
    }

    // Filter, then order, once — the ids are fixed for the enumeration's life (see RecordEnumerationImpl).
    // Without a comparator the order is ascending record id: MIDP leaves it undefined, and insertion
    // order is what a store that only ever adds would give anyway.
    async fn enumerate_records(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        filter: ClassInstanceRef<RecordFilter>,
        comparator: ClassInstanceRef<RecordComparator>,
        keep_updated: bool,
    ) -> JvmResult<ClassInstanceRef<RecordEnumeration>> {
        tracing::debug!("javax.microedition.rms.RecordStore::enumerateRecords({this:?}, {filter:?}, {comparator:?}, {keep_updated})");

        let mut ids = Self::get_database(jvm, context, &this).await?.get_record_ids().await;
        ids.sort_unstable();

        let mut kept: Vec<(i32, ClassInstanceRef<Array<i8>>)> = Vec::with_capacity(ids.len());
        for id in ids {
            let record: ClassInstanceRef<Array<i8>> = jvm
                .invoke_virtual(&this, "javax/microedition/rms/RecordStore", "getRecord", "(I)[B", (id as i32,))
                .await?;
            if !filter.is_null() {
                let matches: bool = jvm
                    .invoke_virtual(&filter, "javax/microedition/rms/RecordFilter", "matches", "([B)Z", (record.clone(),))
                    .await?;
                if !matches {
                    continue;
                }
            }

            // Stable insertion sort through the guest's compare: an element moves ahead only of
            // records that FOLLOW it (compare > 0), so EQUIVALENT records keep id order.
            let mut at = kept.len();
            if !comparator.is_null() {
                while at > 0 {
                    let order: i32 = jvm
                        .invoke_virtual(
                            &comparator,
                            "javax/microedition/rms/RecordComparator",
                            "compare",
                            "([B[B)I",
                            (kept[at - 1].1.clone(), record.clone()),
                        )
                        .await?;
                    if order <= 0 {
                        break;
                    }
                    at -= 1;
                }
            }
            kept.insert(at, (id as i32, record));
        }

        let mut array = jvm.instantiate_array("I", kept.len()).await?;
        jvm.store_array(&mut array, 0, kept.into_iter().map(|(id, _)| id)).await?;

        Ok(jvm
            .new_class(
                "net/wie/RecordEnumerationImpl",
                "(Ljavax/microedition/rms/RecordStore;[I)V",
                (this, array),
            )
            .await?
            .into())
    }

    async fn open_record_store(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        name: ClassInstanceRef<String>,
        create: bool,
    ) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("javax.microedition.rms.RecordStore::openRecordStore({name:?}, {create:?})");

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

    async fn get_database(jvm: &Jvm, context: &mut WieJvmContext, this: &ClassInstanceRef<Self>) -> JvmResult<Box<dyn Database>> {
        let db_name = jvm.get_field(this, "dbName", "Ljava/lang/String;").await?;
        let db_name_str = JavaLangString::to_rust_string(jvm, &db_name).await?;

        let system = context.system();
        let pid = system.pid().to_owned();

        Ok(system.platform().database_repository().open(&db_name_str, &pid).await)
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{Array, ClassInstanceRef, JavaError, Result as JvmResult, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use alloc::{vec, vec::Vec};

    use jvm::Jvm;
    use jvm_class_proto::JavaMethodProto;
    use jvm_types::{ClassAccessFlags, MethodAccessFlags};
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

    use crate::{
        classes::javax::microedition::rms::{RecordComparator, RecordEnumeration, RecordFilter},
        get_protos,
    };

    use super::RecordStore;

    #[test]
    fn delete_record_removes_record_and_rejects_unknown_id() -> Result<()> {
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

            let mut data = jvm.instantiate_array("B", 2).await?;
            jvm.store_array(&mut data, 0, [1i8, 2]).await?;
            let record_id: i32 = jvm
                .invoke_virtual(&store, "javax/microedition/rms/RecordStore", "addRecord", "([BII)I", (data, 0, 2))
                .await?;
            assert_eq!(record_id, 1);

            let count: i32 = jvm
                .invoke_virtual(&store, "javax/microedition/rms/RecordStore", "getNumRecords", "()I", ())
                .await?;
            assert_eq!(count, 1);

            let _: () = jvm
                .invoke_virtual(&store, "javax/microedition/rms/RecordStore", "deleteRecord", "(I)V", (record_id,))
                .await?;
            let count: i32 = jvm
                .invoke_virtual(&store, "javax/microedition/rms/RecordStore", "getNumRecords", "()I", ())
                .await?;
            assert_eq!(count, 0);

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
