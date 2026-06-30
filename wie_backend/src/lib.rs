#![no_std]
extern crate alloc;

mod audio_sink;
pub mod canvas;
mod database;
mod executor;
mod platform;
mod screen;
mod system;
mod task;
mod task_runner;
mod time;

pub use self::{
    audio_sink::AudioSink,
    database::{Database, DatabaseRepository, RecordId},
    executor::{AsyncCallable, AsyncCallableResult},
    platform::{Filesystem, Platform},
    screen::Screen,
    system::{Event, FilesystemOverlay, KeyCode, System},
    task_runner::{DefaultTaskRunner, TaskRunner},
    time::Instant,
};

use alloc::{
    boxed::Box,
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec::Vec,
};

use wie_util::{Result, WieError};

pub trait Emulator {
    fn handle_event(&mut self, event: Event);
    fn tick(&mut self) -> Result<()>;
}

pub struct ProfileSample {
    /// Leaf-first call stack: [pc, lr, lr_prev, ...].
    pub stack: Vec<u32>,
    pub count: u64,
}

/// Called periodically during execution with a batch of samples that the
/// profiler accumulated since the previous flush. The callback also fires once
/// more when the runtime shuts down to drain anything still in the buffer.
pub type ProfileCallback = Box<dyn FnMut(Vec<ProfileSample>) + Send + Sync>;

pub struct Options {
    pub enable_gdbserver: bool,
    pub profile: Option<ProfileCallback>,
}

pub fn extract_zip(zip: &[u8]) -> Result<BTreeMap<String, Vec<u8>>> {
    extern crate std; // XXX

    use std::io::{Cursor, Read};
    use zip::ZipArchive;

    let mut archive = ZipArchive::new(Cursor::new(zip)).map_err(|x| WieError::FatalError(format!("Invalid zip archive: {x}")))?;

    let files: BTreeMap<String, Vec<u8>> = (0..archive.len())
        .filter_map(|x| {
            let mut file = match archive.by_index(x) {
                Ok(file) => file,
                Err(err) => return Some(Err(WieError::FatalError(format!("Failed to read zip entry {x}: {err}")))),
            };
            if !file.is_file() {
                return None;
            }

            let mut data = Vec::new();
            if let Err(err) = file.read_to_end(&mut data) {
                return Some(Err(WieError::FatalError(format!("Failed to read zip entry {}: {err}", file.name()))));
            }

            Some(Ok((file.name().to_string(), data)))
        })
        .collect::<Result<_>>()?;

    Ok(strip_common_wrapper_dir(files))
}

/// Some archives wrap every game file inside a single top-level directory (e.g.
/// `<game-name>/__adf__`, `<game-name>/foo.jar`). The platform detectors and
/// loaders look for markers (`__adf__`, `app_info`, `.msd`) and jar entries at
/// the archive root, so a uniform wrapper directory makes an otherwise valid
/// game look unrecognized. If — and only if — every entry shares the same first
/// path component, strip it so the contents sit at the root. A multi-root
/// archive (no shared prefix) is returned unchanged.
fn strip_common_wrapper_dir(files: BTreeMap<String, Vec<u8>>) -> BTreeMap<String, Vec<u8>> {
    if files.is_empty() {
        return files;
    }

    let first_component = |path: &str| -> Option<String> {
        let idx = path.find('/')?;
        Some(path[..idx].to_string())
    };

    let prefix = match files.keys().next().and_then(|k| first_component(k)) {
        Some(p) => p,
        None => return files, // first entry is at the root already
    };
    if !files.keys().all(|k| first_component(k).as_deref() == Some(prefix.as_str())) {
        return files;
    }

    files
        .into_iter()
        .map(|(path, data)| (path[prefix.len() + 1..].to_string(), data))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: &[&str]) -> BTreeMap<String, Vec<u8>> {
        entries.iter().map(|p| ((*p).to_string(), Vec::new())).collect()
    }

    #[test]
    fn strips_single_wrapper_dir() {
        let out = strip_common_wrapper_dir(map(&["game/__adf__", "game/foo.jar", "game/P/data"]));
        assert!(out.contains_key("__adf__"));
        assert!(out.contains_key("foo.jar"));
        assert!(out.contains_key("P/data"));
    }

    #[test]
    fn leaves_root_level_files_untouched() {
        let out = strip_common_wrapper_dir(map(&["__adf__", "foo.jar"]));
        assert!(out.contains_key("__adf__"));
        assert!(out.contains_key("foo.jar"));
    }

    #[test]
    fn keeps_multi_root_archive() {
        // No shared first component: must not strip anything.
        let out = strip_common_wrapper_dir(map(&["a/__adf__", "b/foo.jar"]));
        assert!(out.contains_key("a/__adf__"));
        assert!(out.contains_key("b/foo.jar"));
    }

    #[test]
    fn keeps_mixed_root_and_dir() {
        // A root-level marker alongside a directory must not be stripped.
        let out = strip_common_wrapper_dir(map(&["__adf__", "P/data"]));
        assert!(out.contains_key("__adf__"));
        assert!(out.contains_key("P/data"));
    }
}
