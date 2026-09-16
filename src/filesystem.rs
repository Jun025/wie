use std::{
    fs::{self, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Component, Path, PathBuf},
};

use directories::ProjectDirs;

use wie_backend::Filesystem;

/// Persistent filesystem backed by `std::fs` under `<base>/<aid>/fs/<path>`.
/// Any I/O error or rejected path returns the trait's failure value.
pub struct CliFilesystem {
    base_path: PathBuf,
}

impl CliFilesystem {
    pub fn new() -> Self {
        let base_dir = ProjectDirs::from("net", "dlunch", "wie").unwrap();
        Self {
            base_path: base_dir.data_dir().to_owned(),
        }
    }

    fn path_for(&self, aid: &str, path: &str) -> Option<PathBuf> {
        let sanitized_aid: String = aid.chars().filter(|c| !matches!(c, '/' | '\\' | '\0')).collect();
        if sanitized_aid.is_empty() || sanitized_aid == "." || sanitized_aid == ".." {
            tracing::error!(aid, path, "rejected: invalid aid");
            return None;
        }

        let mut normalized = PathBuf::new();
        for component in Path::new(path).components() {
            match component {
                Component::Normal(c) => normalized.push(c),
                Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    tracing::error!(aid, path, "path traversal attempt rejected");
                    return None;
                }
            }
        }

        if normalized.as_os_str().is_empty() {
            tracing::error!(aid, path, "rejected: empty normalized path");
            return None;
        }

        Some(self.base_path.join(&sanitized_aid).join("fs").join(normalized))
    }
}

impl Default for CliFilesystem {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Filesystem for CliFilesystem {
    async fn exists(&self, aid: &str, path: &str) -> bool {
        let Some(disk_path) = self.path_for(aid, path) else {
            return false;
        };

        match disk_path.metadata() {
            Ok(md) => md.is_file(),
            Err(_) => false,
        }
    }

    async fn size(&self, aid: &str, path: &str) -> Option<usize> {
        let disk_path = self.path_for(aid, path)?;
        let md = disk_path.metadata().ok()?;
        if !md.is_file() {
            return None;
        }
        Some(md.len() as usize)
    }

    async fn read(&self, aid: &str, path: &str, offset: usize, count: usize, buf: &mut [u8]) -> Option<usize> {
        let disk_path = self.path_for(aid, path)?;

        let mut file = match OpenOptions::new().read(true).open(&disk_path) {
            Ok(f) => f,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    return None;
                }
                tracing::warn!(aid, path, error = %err, "read: open failed");
                return None;
            }
        };

        let size = file.metadata().map(|m| m.len() as usize).unwrap_or(0);
        if offset >= size {
            return Some(0);
        }

        if let Err(err) = file.seek(SeekFrom::Start(offset as u64)) {
            tracing::warn!(aid, path, error = %err, "read: seek failed");
            return Some(0);
        }

        let to_read = core::cmp::min(count, size - offset);
        let slice = &mut buf[..to_read];
        // read_exact so short reads surface to the caller only at EOF, not
        // on signal interruption.
        match file.read_exact(slice) {
            Ok(()) => Some(to_read),
            Err(err) => {
                tracing::warn!(aid, path, error = %err, "read: IO error");
                Some(0)
            }
        }
    }

    async fn write(&self, aid: &str, path: &str, offset: usize, data: &[u8]) -> usize {
        let Some(disk_path) = self.path_for(aid, path) else {
            return 0;
        };

        if let Some(parent) = disk_path.parent()
            && let Err(err) = fs::create_dir_all(parent)
        {
            tracing::warn!(aid, path, error = %err, "write: create parent dir failed");
            return 0;
        }

        let mut file = match OpenOptions::new().read(true).write(true).create(true).truncate(false).open(&disk_path) {
            Ok(f) => f,
            Err(err) => {
                tracing::warn!(aid, path, error = %err, "write: open failed");
                return 0;
            }
        };

        let current_size = file.metadata().map(|m| m.len() as usize).unwrap_or(0);

        if offset > current_size {
            // OS sparse extend avoids host allocation for the gap; POSIX
            // ftruncate and Windows SetEndOfFile both zero-fill the
            // newly-created region.
            if let Err(err) = file.set_len(offset as u64) {
                tracing::warn!(aid, path, error = %err, "write: set_len extend failed");
                return 0;
            }
        }

        if let Err(err) = file.seek(SeekFrom::Start(offset as u64)) {
            tracing::warn!(aid, path, error = %err, "write: seek failed");
            return 0;
        }

        match file.write_all(data) {
            Ok(()) => data.len(),
            Err(err) => {
                tracing::warn!(aid, path, error = %err, "write: write_all failed");
                0
            }
        }
    }

    async fn truncate(&self, aid: &str, path: &str, len: usize) {
        let Some(disk_path) = self.path_for(aid, path) else {
            return;
        };

        if let Some(parent) = disk_path.parent()
            && let Err(err) = fs::create_dir_all(parent)
        {
            tracing::warn!(aid, path, error = %err, "truncate: create parent dir failed");
            return;
        }

        let file = match OpenOptions::new().read(true).write(true).create(true).truncate(false).open(&disk_path) {
            Ok(f) => f,
            Err(err) => {
                tracing::warn!(aid, path, error = %err, "truncate: open failed");
                return;
            }
        };

        if let Err(err) = file.set_len(len as u64) {
            tracing::warn!(aid, path, error = %err, "truncate: set_len failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::CliFilesystem;

    /// `path_for` is the whole security boundary of this backend, and until now
    /// nothing exercised it: measured 2026-09-07, this file had **zero** tests.
    /// It is also one half of the only host-divergent pair in the resource path —
    /// `wie_featurephone::WebFilesystem::key` normalizes the same guest strings by hand,
    /// and the two disagree (see `differs_from_web_on_absolute_and_backslash`).
    fn fs() -> CliFilesystem {
        CliFilesystem {
            base_path: PathBuf::from("/base"),
        }
    }

    #[test]
    fn normal_path_lands_under_aid() {
        let got = fs().path_for("game", "save/slot1.dat").unwrap();
        assert_eq!(got, PathBuf::from("/base/game/fs/save/slot1.dat"));
    }

    #[test]
    fn cur_dir_segments_are_dropped() {
        let got = fs().path_for("game", "./save/./slot1.dat").unwrap();
        assert_eq!(got, PathBuf::from("/base/game/fs/save/slot1.dat"));
    }

    #[test]
    fn traversal_and_empty_are_rejected() {
        // The failure this boundary exists for: escaping the per-aid directory.
        assert_eq!(fs().path_for("game", "../other/fs/save.dat"), None);
        assert_eq!(fs().path_for("game", "save/../../escape"), None);
        // Nothing left after normalization is a rejection, not an empty join.
        assert_eq!(fs().path_for("game", ""), None);
        assert_eq!(fs().path_for("game", "."), None);
    }

    #[test]
    fn unsafe_aid_is_rejected() {
        // `aid` is filtered, not normalized: separators are stripped, and what is
        // left must still be a usable directory name.
        assert_eq!(fs().path_for("", "a"), None);
        assert_eq!(fs().path_for("/", "a"), None);
        assert_eq!(fs().path_for("..", "a"), None);
        assert_eq!(fs().path_for("a/b", "x").unwrap(), PathBuf::from("/base/ab/fs/x"));
    }

    /// The divergence the resource fallback would expose, pinned as a fact rather
    /// than an inference. `FilesystemOverlay::normalize_guest_path` currently masks
    /// both cases — it strips leading `/` and rejects any `\` before either backend
    /// sees the string — so today neither reaches here. That masking is the only
    /// reason the two backends agreeing does not matter; if it is ever relaxed,
    /// this test says what changes.
    #[test]
    fn differs_from_web_on_absolute_and_backslash() {
        // Absolute: CLI rejects (Component::RootDir), WebFilesystem::key accepts and
        // yields "save.dat" because splitting on '/' just produces a leading "".
        assert_eq!(fs().path_for("game", "/save.dat"), None);

        // Backslash: on unix `Path::components` treats it as an ordinary character,
        // so this is ONE filename; WebFilesystem::key splits on it and yields "a/b".
        #[cfg(unix)]
        assert_eq!(fs().path_for("game", "a\\b").unwrap(), PathBuf::from("/base/game/fs/a\\b"));
    }
}
