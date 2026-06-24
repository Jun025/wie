use alloc::sync::Arc;
use std::collections::BTreeMap;
use std::sync::Mutex;

use wie_backend::Filesystem;

/// Shared, app-isolated file store. Keyed by `(aid, normalized_path)`.
///
/// Lives entirely in browser memory for the session. The JS layer can snapshot
/// it to IndexedDB and restore it on the next visit (see `lib::export_fs` /
/// `lib::import_fs`); game files are never part of this store — only the saves
/// the guest app writes through the `Filesystem` API.
pub type FsStore = Arc<Mutex<BTreeMap<(String, String), Vec<u8>>>>;

pub struct WebFilesystem {
    store: FsStore,
}

impl WebFilesystem {
    pub fn new(store: FsStore) -> Self {
        Self { store }
    }

    /// Normalize a guest path the same way the CLI filesystem does: reject
    /// traversal / absolute components, drop `.` segments. Returns `None` when
    /// the aid or path is unsafe.
    fn key(&self, aid: &str, path: &str) -> Option<(String, String)> {
        let sanitized_aid: String = aid.chars().filter(|c| !matches!(c, '/' | '\\' | '\0')).collect();
        if sanitized_aid.is_empty() || sanitized_aid == "." || sanitized_aid == ".." {
            return None;
        }

        let mut normalized: Vec<&str> = Vec::new();
        for segment in path.split(['/', '\\']) {
            match segment {
                "" | "." => {}
                ".." => return None,
                seg => normalized.push(seg),
            }
        }
        if normalized.is_empty() {
            return None;
        }

        Some((sanitized_aid, normalized.join("/")))
    }
}

#[async_trait::async_trait]
impl Filesystem for WebFilesystem {
    async fn exists(&self, aid: &str, path: &str) -> bool {
        let Some(key) = self.key(aid, path) else {
            return false;
        };
        self.store.lock().unwrap().contains_key(&key)
    }

    async fn size(&self, aid: &str, path: &str) -> Option<usize> {
        let key = self.key(aid, path)?;
        self.store.lock().unwrap().get(&key).map(|d| d.len())
    }

    async fn read(&self, aid: &str, path: &str, offset: usize, count: usize, buf: &mut [u8]) -> Option<usize> {
        let key = self.key(aid, path)?;
        let store = self.store.lock().unwrap();
        let data = store.get(&key)?;

        if offset >= data.len() {
            return Some(0);
        }

        let to_read = core::cmp::min(count, data.len() - offset);
        buf[..to_read].copy_from_slice(&data[offset..offset + to_read]);
        Some(to_read)
    }

    async fn write(&self, aid: &str, path: &str, offset: usize, data: &[u8]) -> usize {
        let Some(key) = self.key(aid, path) else {
            tracing::warn!(aid, path, "write rejected: invalid path");
            return 0;
        };

        let mut store = self.store.lock().unwrap();
        let file = store.entry(key).or_default();

        let end = offset + data.len();
        if end > file.len() {
            file.resize(end, 0);
        }
        file[offset..end].copy_from_slice(data);

        data.len()
    }

    async fn truncate(&self, aid: &str, path: &str, len: usize) {
        let Some(key) = self.key(aid, path) else {
            return;
        };

        let mut store = self.store.lock().unwrap();
        let file = store.entry(key).or_default();
        file.resize(len, 0);
    }
}
