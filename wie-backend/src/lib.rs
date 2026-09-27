#![no_std]
extern crate alloc;

mod audio_sink;
pub mod canvas;
mod database;
mod executor;
mod frame_pacer;
mod pacing;
mod platform;
mod screen;
mod system;
mod task;
mod task_runner;
pub mod text_layout;
mod time;

pub use self::{
    audio_sink::{AudioCommand, AudioEventData, AudioHandle, AudioSequence, AudioSink, TimedAudioEvent},
    canvas::Font,
    database::{Database, DatabaseRepository, RecordId},
    executor::{AsyncCallable, AsyncCallableResult, TICK_BUDGET_MS},
    frame_pacer::FramePacer,
    pacing::Pacing,
    platform::{Filesystem, Platform},
    screen::Screen,
    system::{Event, FilesystemOverlay, KeyCode, System},
    task::YieldFuture,
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

    /// One host frame's worth of emulation at the default budget (`TICK_BUDGET_MS`). Hosts that
    /// know their frame interval call `tick_for` instead.
    fn tick(&mut self) -> Result<()> {
        self.tick_for(TICK_BUDGET_MS)
    }

    /// Run the emulator for at most `budget_ms` of wall-clock time (plus one poll's overrun).
    fn tick_for(&mut self, budget_ms: u64) -> Result<()>;

    /// The engine's pacing counters since the last call (`Pacing`).
    fn take_pacing(&mut self) -> Pacing;
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

    Ok(strip_common_wrapper_dir(reroot_at_marker(files)))
}

/// A game archive's marker (`__adf__`, `app_info`, `*.msd`) sits at the root. When none does but
/// exactly one directory holds one, that directory IS the game: re-root there and drop what lies
/// outside it. Measured 2026-09-27: 3 KTF titles nest the game two or three levels deep, two of
/// them next to a stray screenshot at the root, which the single-wrapper strip below cannot reach.
fn reroot_at_marker(files: BTreeMap<String, Vec<u8>>) -> BTreeMap<String, Vec<u8>> {
    let is_marker = |path: &str| {
        let name = path.rsplit('/').next().unwrap_or(path);
        name == "__adf__" || name == "app_info" || name.ends_with(".msd")
    };
    if files.keys().any(|k| !k.contains('/') && is_marker(k)) {
        return files;
    }

    let mut dirs = files.keys().filter(|k| is_marker(k)).map(|k| &k[..=k.rfind('/').unwrap()]);
    let Some(dir) = dirs.next().map(str::to_string) else {
        return files;
    };
    if dirs.any(|d| d != dir) {
        return files;
    }

    files
        .into_iter()
        .filter_map(|(path, data)| path.strip_prefix(&dir).map(|p| (p.to_string(), data)))
        .collect()
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
    fn reroots_at_the_one_nested_marker_dir() {
        let out = reroot_at_marker(map(&["W/apps/x/__adf__", "W/apps/x/x.jar", "shot.png"]));
        assert_eq!(out.keys().collect::<Vec<_>>(), ["__adf__", "x.jar"]);
        // Root marker, or two candidate games: unchanged.
        assert!(reroot_at_marker(map(&["__adf__", "d/app_info"])).contains_key("d/app_info"));
        assert!(reroot_at_marker(map(&["a/__adf__", "b/__adf__"])).contains_key("a/__adf__"));
    }

    #[test]
    fn extract_zip_reroots_a_nested_game() {
        extern crate std;
        use std::io::{Cursor, Write};

        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let stored = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for name in ["shot.png", "g/g-wipi1.2/__adf__", "g/g-wipi1.2/g.jar"] {
            zip.start_file(name, stored).unwrap();
            zip.write_all(b"x").unwrap();
        }
        let files = extract_zip(&zip.finish().unwrap().into_inner()).unwrap();
        assert!(files.contains_key("__adf__") && files.contains_key("g.jar"));
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
