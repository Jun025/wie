use alloc::sync::Arc;
use std::collections::BTreeMap;
use std::sync::Mutex;

use wie_backend::{Database, DatabaseRepository, RecordId, System};

/// Shared record store, keyed by `(app_id, db_name)` → records.
///
/// Same lifecycle as the filesystem store: in-memory for the session, with the
/// JS layer responsible for IndexedDB persistence. Holds only guest-written
/// save records, never game files.
pub type DbStore = Arc<Mutex<BTreeMap<(String, String), BTreeMap<RecordId, Vec<u8>>>>>;

pub struct WebDatabaseRepository {
    store: DbStore,
}

impl WebDatabaseRepository {
    pub fn new(store: DbStore) -> Self {
        Self { store }
    }

    fn key(name: &str, app_id: &str) -> (String, String) {
        let sanitized_app_id: String = app_id.chars().filter(|c| !matches!(c, '/' | '\\' | '\0')).collect();
        let app_id = if sanitized_app_id.is_empty() || sanitized_app_id == "." || sanitized_app_id == ".." {
            "_".to_owned()
        } else {
            sanitized_app_id
        };
        (app_id, name.to_owned())
    }
}

#[async_trait::async_trait]
impl DatabaseRepository for WebDatabaseRepository {
    async fn open(&self, _system: &System, name: &str, app_id: &str) -> Box<dyn Database> {
        let key = Self::key(name, app_id);
        self.store.lock().unwrap().entry(key.clone()).or_default();
        Box::new(WebDatabase {
            store: self.store.clone(),
            key,
        })
    }

    async fn exists(&self, _system: &System, name: &str, app_id: &str) -> bool {
        self.store.lock().unwrap().contains_key(&Self::key(name, app_id))
    }

    async fn delete(&self, _system: &System, name: &str, app_id: &str) -> bool {
        self.store.lock().unwrap().remove(&Self::key(name, app_id)).is_some()
    }
}

pub struct WebDatabase {
    store: DbStore,
    key: (String, String),
}

impl WebDatabase {
    fn find_empty_record_id(records: &BTreeMap<RecordId, Vec<u8>>) -> RecordId {
        // XXX midp requires the first record to be 1
        let mut id = 1;
        while records.contains_key(&id) {
            id += 1;
        }
        id
    }
}

#[async_trait::async_trait]
impl Database for WebDatabase {
    async fn next_id(&self) -> RecordId {
        let store = self.store.lock().unwrap();
        let records = store.get(&self.key).expect("database opened before use");
        Self::find_empty_record_id(records)
    }

    async fn add(&mut self, data: &[u8]) -> RecordId {
        let mut store = self.store.lock().unwrap();
        let records = store.entry(self.key.clone()).or_default();
        let id = Self::find_empty_record_id(records);
        records.insert(id, data.to_vec());
        id
    }

    async fn get(&self, id: RecordId) -> Option<Vec<u8>> {
        let store = self.store.lock().unwrap();
        store.get(&self.key)?.get(&id).cloned()
    }

    async fn set(&mut self, id: RecordId, data: &[u8]) -> bool {
        let mut store = self.store.lock().unwrap();
        let records = store.entry(self.key.clone()).or_default();
        records.insert(id, data.to_vec());
        true
    }

    async fn delete(&mut self, id: RecordId) -> bool {
        let mut store = self.store.lock().unwrap();
        match store.get_mut(&self.key) {
            Some(records) => records.remove(&id).is_some(),
            None => false,
        }
    }

    async fn get_record_ids(&self) -> Vec<RecordId> {
        let store = self.store.lock().unwrap();
        store.get(&self.key).map(|r| r.keys().copied().collect()).unwrap_or_default()
    }
}
