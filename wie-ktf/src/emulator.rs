use core::{mem::size_of, pin::Pin, task::Poll};

use alloc::{borrow::ToOwned, boxed::Box, collections::BTreeMap, format, string::String, vec, vec::Vec};

use bytemuck::Zeroable;
use futures::future::poll_fn;
use jvm::{ClassInstance, Result as JvmResult, runtime::JavaLangString};

use wie_backend::{Emulator, Event, Options, Pacing, Platform, System, TaskRunner};
use wie_core_arm::{Allocator, ArmCore};
use wie_jvm_support::JvmSupport;
use wie_util::{Result, WieError, write_generic};

use crate::{
    adf::{KtfAdf, find_client_bin},
    runtime::{KtfJvmSupport, KtfJvmThreadContext},
};

pub const IMAGE_BASE: u32 = 0x100000;

fn is_clet_mode(main_class_name: &str) -> bool {
    main_class_name == "Clet"
}

// The jar is normally `<AID>.jar`, but some archives carry their only jar under another id
// (the ADF was re-issued with a new AID and the jar kept its original name). Fall back to the
// sole `.jar` then; with zero or several jars keep `<AID>.jar` and let the load report it.
fn jar_filename(aid: &str, files: &BTreeMap<String, Vec<u8>>) -> String {
    let expected = format!("{aid}.jar");
    if files.contains_key(&expected) {
        return expected;
    }
    let mut jars = files.keys().filter(|name| name.ends_with(".jar"));
    match (jars.next(), jars.next()) {
        (Some(only), None) => only.clone(),
        _ => expected,
    }
}

/// Packaged saves written by the original owner's handset that no other host can load, keyed by the
/// MD5 of the title's jar (the jar identifies the build, as in `wie-lgt`'s `ORPHAN_ENTRIES`).
/// Deliberately a list, never a rule — most KTF archives ship saves that load fine, and dropping
/// every `P/` save would change their start state. A dropped title boots as a fresh install does:
/// with no save, and writes its own.
const DEVICE_BOUND_SAVES: &[([u8; 16], &[&str])] = &[
    // AID 0103451A (archive sha256 59263295de74…). `res/save.sav` +0x214 holds 0x01d14250, a heap
    // pointer of the handset that wrote it: the game reads 192 bytes into a struct, then
    // `stream_read`s 920 bytes to the struct's +0x30 — that stale pointer (docs/report/0398).
    // `res/savem.sav` came from the same handset and goes with it.
    (
        [
            0x69, 0x7c, 0xcf, 0x63, 0x8d, 0xf0, 0x4b, 0x98, 0x27, 0xdb, 0x67, 0x75, 0x34, 0x27, 0xd5, 0x8e,
        ],
        &["res/save.sav", "res/savem.sav"],
    ),
];

/// The original owner's settings, shipped in the archive, that start the title with its sound off.
/// Measured per title, keyed like `DEVICE_BOUND_SAVES` and dropped the same way, so the title starts
/// as a fresh install does: removing only these files brings the sound back in the 30 s probes
/// (0 plays → 1–8), and on the two titles whose settings screen was opened, turning the sound up there
/// does the same — the game honours them, nothing in the engine is silent (docs/report/0417).
/// Deliberately a list: other shipped settings and saves load as they always have.
const OWNER_SOUND_OFF_SETTINGS: &[([u8; 16], &[&str])] = &[
    // 2fc792485d91: one 11,255-byte record: settings and progress together.
    (
        [
            0x26, 0xc9, 0x60, 0x35, 0x44, 0x6a, 0x98, 0x31, 0xcd, 0x3f, 0x53, 0xf7, 0x98, 0x48, 0x74, 0x30,
        ],
        &["sp.dat.db", "sp.dat.idx"],
    ),
    // 3185174d2121: 7 bytes.
    (
        [
            0xe9, 0x18, 0x85, 0xfb, 0x2c, 0x0f, 0xec, 0x9e, 0xa9, 0x7a, 0x0d, 0x29, 0xc1, 0x5e, 0xd7, 0x2f,
        ],
        &["setup.dat"],
    ),
    // 5e53e490c6f1: 12 bytes; its settings screen shows the sound level at 0.
    (
        [
            0x1a, 0x21, 0x23, 0xeb, 0x97, 0xe2, 0x51, 0xdb, 0xcb, 0x4b, 0x49, 0x0c, 0xee, 0x86, 0x30, 0xea,
        ],
        &["opt.txt"],
    ),
    // 6a885f89343c: a 6-byte DB; the title's other three DBs are not it.
    (
        [
            0x52, 0xfd, 0x30, 0xf7, 0x1d, 0x56, 0x67, 0x96, 0x2a, 0xa0, 0xab, 0x99, 0xf4, 0x70, 0x68, 0xb0,
        ],
        &["sky4.db", "sky4.idx"],
    ),
    // bfa8ec352451: one record: the owner's name, scores and option flags.
    (
        [
            0xcf, 0xa9, 0xe1, 0x86, 0x07, 0xf6, 0xdc, 0x07, 0xc6, 0x54, 0x82, 0x06, 0xbd, 0x25, 0xc2, 0xdf,
        ],
        &["kill.db", "kill.idx"],
    ),
    // de00506611a5: four records.
    (
        [
            0x05, 0x55, 0x29, 0x80, 0x3d, 0x08, 0xea, 0x81, 0x6b, 0xa2, 0x12, 0x2a, 0x7f, 0x8d, 0xa6, 0x87,
        ],
        &["haga2.db", "haga2.idx"],
    ),
    // 7218e8720f8c: 8 bytes among the data files; the title writes 8 bytes back when it is absent.
    (
        [
            0x93, 0x82, 0xc5, 0x4f, 0x44, 0xd6, 0xdc, 0xea, 0x90, 0xa9, 0x09, 0x51, 0x6c, 0x94, 0x2a, 0xc6,
        ],
        &["12_0.ida"],
    ),
    // 8bfd08fe4370: 18 bytes; the title writes 18 bytes back when it is absent.
    (
        [
            0x01, 0xe6, 0x86, 0xc6, 0xfb, 0x5a, 0x46, 0xde, 0x39, 0xb3, 0x7e, 0xa4, 0x18, 0x51, 0x9e, 0x3b,
        ],
        &["kjik.mcf"],
    ),
    // 34ab350dc98a: 3 bytes; the second is the sound level and the owner's is 0 (set to 3, it plays).
    (
        [
            0x1c, 0x40, 0xaa, 0x0b, 0x3c, 0x04, 0xa7, 0x60, 0x95, 0xaa, 0x68, 0x6c, 0x28, 0xcf, 0xd0, 0x46,
        ],
        &["BG0"],
    ),
    // c7f543c73b91: four ints; the first is the sound level and the owner's is 0 (set to 3, it plays).
    (
        [
            0x75, 0x09, 0x2c, 0xa0, 0x24, 0xa3, 0x54, 0x66, 0x08, 0xca, 0x10, 0x47, 0xbd, 0x9d, 0x30, 0x06,
        ],
        &["option.dat"],
    ),
    // db8ef04a6504: a 6-byte DB; the owner's first byte is 0, the title writes 3 there when it is absent.
    (
        [
            0x64, 0x87, 0x7e, 0x4b, 0x6d, 0x24, 0x50, 0x2a, 0xb8, 0xdb, 0x3e, 0xb0, 0x5b, 0x6f, 0x48, 0xb2,
        ],
        &["config"],
    ),
    // e085e193211d: 8 bytes; the first is the sound level and the owner's is 0 (set to 5, it plays).
    (
        [
            0x08, 0xa2, 0xe4, 0x6b, 0xe3, 0x74, 0x81, 0x68, 0xcc, 0x0b, 0x00, 0x21, 0x5e, 0x57, 0x87, 0x90,
        ],
        &["ga/cf.ga"],
    ),
    // 36b82cb67723: the profile and its backup copy — the owner's name, and a first int that is the
    // sound level, 0 (set to 3, it plays). The other eight `peng*` files are not it.
    (
        [
            0x47, 0xe3, 0x24, 0xc4, 0x31, 0x55, 0x49, 0xd8, 0x04, 0x7a, 0x5b, 0xfc, 0xbb, 0xf1, 0xac, 0x19,
        ],
        &["peng0.txt", "pengB0.txt"],
    ),
];

/// The original owner's progress, shipped in the archive, that «continue» opens: a new player would
/// pick up someone else's game. Measured per title — the original and a copy without only these files,
/// same keys — and kept to the titles where «continue» was seen to open it; the copy starts as a fresh
/// install does (the menu's default moves to «new», or «continue» says there is no save, and the new
/// game runs). Deliberately a list (docs/report/0419): an archive mixes progress with settings, game
/// data and download-complete flags, and dropping those stops titles that 0416 made playable.
const OWNER_PROGRESS_SAVES: &[([u8; 16], &[&str])] = &[
    // 3185174d2121: the owner's hospital: «continue» opens it with nine stats at Lv9.
    (
        [
            0xe9, 0x18, 0x85, 0xfb, 0x2c, 0x0f, 0xec, 0x9e, 0xa9, 0x7a, 0x0d, 0x29, 0xc1, 0x5e, 0xd7, 0x2f,
        ],
        &["hospital1.dat", "hospital2.dat"],
    ),
    // 3b82763edba8: «continue» opens the owner's house with 39,395,782 in hand; dropped, it says there is no save.
    (
        [
            0x8c, 0x80, 0x40, 0x67, 0x86, 0x37, 0xea, 0xf3, 0x20, 0xe4, 0x43, 0x27, 0x0e, 0xc9, 0xc1, 0xc0,
        ],
        &["real.db", "real.idx"],
    ),
    // 4bcd17980c05: «continue» opens the owner's run in the desert with 300 in hand.
    (
        [
            0x59, 0xca, 0xba, 0xbf, 0x2c, 0x20, 0xd6, 0x91, 0x43, 0x56, 0xd7, 0x92, 0xa4, 0x78, 0x28, 0x3f,
        ],
        &["LdmDataFile.dat"],
    ),
    // 4df05a4dc452: «continue» opens the owner's game with 10 coins; the config and doll files stay.
    (
        [
            0xd1, 0xdb, 0x89, 0x33, 0x26, 0x24, 0xee, 0xf4, 0x67, 0xec, 0xd8, 0xb6, 0x6a, 0xee, 0xcd, 0xdb,
        ],
        &["GameScore.txt"],
    ),
    // 5e53e490c6f1: «continue» opens the owner's year 196 with 15 players and 500M (its settings are in `OWNER_SOUND_OFF_SETTINGS`).
    (
        [
            0x1a, 0x21, 0x23, 0xeb, 0x97, 0xe2, 0x51, 0xdb, 0xcb, 0x4b, 0x49, 0x0c, 0xee, 0x86, 0x30, 0xea,
        ],
        &["save0.txt"],
    ),
    // 6a885f89343c: the owner's two save slots and the slot index (scores 1720 and 1980).
    (
        [
            0x52, 0xfd, 0x30, 0xf7, 0x1d, 0x56, 0x67, 0x96, 0x2a, 0xa0, 0xab, 0x99, 0xf4, 0x70, 0x68, 0xb0,
        ],
        &["sky0.db", "sky0.idx", "sky2.db", "sky2.idx", "sky3.db", "sky3.idx"],
    ),
    // 6b51d12b4be6: «continue» opens the owner's story mode; dropped, it says there is no save.
    (
        [
            0xaf, 0x3f, 0x39, 0xb6, 0x04, 0x56, 0x5b, 0x81, 0x16, 0x06, 0x5a, 0xf5, 0x7c, 0xf5, 0x58, 0x11,
        ],
        &["hotong.dat"],
    ),
    // b1ec149b354c: «continue» opens the owner's 26 May accounts; the pizza, taste and topping tables are game data and stay.
    (
        [
            0x36, 0xaf, 0x8b, 0x87, 0xd8, 0x07, 0x46, 0x46, 0xbe, 0xfb, 0x64, 0x58, 0xce, 0x0f, 0x15, 0xff,
        ],
        &["gGameData.db", "gGameData.idx"],
    ),
    // e09aca27c132: «continue» opens the owner's season map with 8.88M; the fortune table stays.
    (
        [
            0x3f, 0x5c, 0x5e, 0x5a, 0x45, 0x21, 0x87, 0x05, 0x95, 0xc9, 0xbb, 0xfc, 0xa7, 0x82, 0xe8, 0x06,
        ],
        &["jsydata.db", "jsydata.idx"],
    ),
    // eb1614abed70: «continue» opens the owner's FILE 1 in the village; the 1-byte `certify`/`mix` flags stay.
    (
        [
            0x6a, 0x67, 0xa9, 0x86, 0xb4, 0x61, 0x1e, 0x52, 0x65, 0xe3, 0xd1, 0x7d, 0x05, 0x01, 0xa6, 0xcd,
        ],
        &["save0", "save0_crc"],
    ),
    // edd11ce9b8d7: «continue» opens the owner's scenario with the beginner course passed.
    (
        [
            0x3d, 0xfd, 0xc1, 0x4f, 0x9b, 0xbc, 0xda, 0x85, 0xed, 0xfd, 0x06, 0x39, 0xe7, 0x2c, 0x8b, 0x40,
        ],
        &["io.dat"],
    ),
];

/// Remove the files `table` names for this jar, bare and under either private directory spelling
/// (`load` mounts all three at the same path).
fn drop_device_bound_saves(files: &mut BTreeMap<String, Vec<u8>>, jar: &[u8], table: &[([u8; 16], &[&str])]) {
    let hash = md5::compute(jar).0;
    for (_, names) in table.iter().filter(|(md5, _)| *md5 == hash) {
        for name in *names {
            for key in [(*name).to_owned(), format!("P/{name}"), format!("p/{name}")] {
                if files.remove(&key).is_some() {
                    tracing::info!("Not mounting the original owner's {key}");
                }
            }
        }
    }
}

/// The archive's private directory is mounted at the root, where the title looks for it. Archives
/// spell it `P/` or `p/`: 25 KTF titles ship theirs lowercase, and one of them (1793f87924d4) shows
/// «downloading 1/11» forever because `isFile` misses the data it shipped under `p/`
/// (docs/report/0414).
fn private_path(path: &str) -> &str {
    path.strip_prefix("P/").or_else(|| path.strip_prefix("p/")).unwrap_or(path)
}

struct KtfTaskRunner {
    core: ArmCore,
}

#[async_trait::async_trait]
impl TaskRunner for KtfTaskRunner {
    async fn run(&self, mut future: Pin<Box<dyn Future<Output = Result<()>> + Send>>) -> Result<()> {
        let mut core = self.core.clone();
        let ptr_thread_context = Allocator::alloc(&mut core, size_of::<KtfJvmThreadContext>() as u32)?;
        write_generic(&mut core, ptr_thread_context, KtfJvmThreadContext::zeroed())?;

        let mut poll_core = self.core.clone();
        let result = self
            .core
            .run_in_thread(move || {
                poll_fn(move |context| {
                    // KTF's first native init argument points at this cell and dereferences it for each stack check and try block.
                    if let Err(error) = KtfJvmSupport::set_current_thread_context(&mut poll_core, ptr_thread_context) {
                        return Poll::Ready(Err(error));
                    }

                    future.as_mut().poll(context)
                })
            })?
            .await;

        Allocator::free(&mut core, ptr_thread_context, size_of::<KtfJvmThreadContext>() as u32)?;

        result
    }
}

pub struct KtfEmulator {
    core: ArmCore,
    system: System,
}

impl KtfEmulator {
    pub fn from_archive(platform: Box<dyn Platform>, mut files: BTreeMap<String, Vec<u8>>, options: Options) -> Result<Self> {
        let adf = files
            .get("__adf__")
            .ok_or_else(|| WieError::FatalError("Missing __adf__ in KTF archive".into()))?;
        let adf = KtfAdf::parse(adf);

        tracing::info!("Loading app {}, pid {}, mclass {}", adf.aid, adf.pid, adf.mclass);
        if let Some((width, height)) = adf.display_size
            && let Err(error) = platform.screen().resize(width, height)
        {
            tracing::warn!("Ignoring unsupported display size {width}x{height}: {error}");
        }

        let jar_filename = jar_filename(&adf.aid, &files);
        if let Some(jar) = files.get(&jar_filename).cloned() {
            drop_device_bound_saves(&mut files, &jar, DEVICE_BOUND_SAVES);
            drop_device_bound_saves(&mut files, &jar, OWNER_SOUND_OFF_SETTINGS);
            drop_device_bound_saves(&mut files, &jar, OWNER_PROGRESS_SAVES);
        }

        Self::load(platform, &jar_filename, &adf.pid, &adf.aid, Some(adf.mclass), &files, options)
    }

    pub fn from_jar(
        platform: Box<dyn Platform>,
        jar_filename: &str,
        jar: Vec<u8>,
        pid: &str,
        aid: &str,
        main_class_name: Option<String>,
        options: Options,
    ) -> Result<Self> {
        let files = [(jar_filename.to_owned(), jar)].into_iter().collect();

        Self::load(platform, jar_filename, pid, aid, main_class_name, &files, options)
    }

    pub fn loadable_archive(files: &BTreeMap<String, Vec<u8>>) -> bool {
        files.contains_key("__adf__")
    }

    pub fn archive_title(files: &BTreeMap<String, Vec<u8>>) -> Option<String> {
        let title = KtfAdf::parse(files.get("__adf__")?).name;
        (!title.is_empty()).then_some(title)
    }

    pub fn archive_id(files: &BTreeMap<String, Vec<u8>>) -> Option<String> {
        let id = KtfAdf::parse(files.get("__adf__")?).pid;
        (!id.is_empty()).then_some(id)
    }

    pub fn archive_icon(files: &BTreeMap<String, Vec<u8>>) -> Option<Vec<u8>> {
        files.get("big.icon").cloned()
    }

    pub fn loadable_jar(jar: &[u8]) -> bool {
        find_client_bin(jar).is_ok()
    }

    fn load(
        platform: Box<dyn Platform>,
        jar_filename: &str,
        pid: &str,
        aid: &str,
        main_class_name: Option<String>,
        files: &BTreeMap<String, Vec<u8>>,
        mut options: Options,
    ) -> Result<Self> {
        let mut core = ArmCore::new(options.enable_gdbserver, options.profile.take())?;
        let system = System::new(platform, pid, aid, KtfTaskRunner { core: core.clone() });

        for (path, data) in files {
            system.filesystem().add_virtual(private_path(path), data.clone());
        }

        Allocator::init(&mut core)?;

        let mut core_clone = core.clone();
        let mut system_clone = system.clone();
        let jar_filename_clone = jar_filename.to_owned();
        system.spawn(async move || Self::start(&mut core_clone, &mut system_clone, jar_filename_clone, main_class_name).await);

        Ok(Self { core, system })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn start(core: &mut ArmCore, system: &mut System, jar_filename: String, main_class_name: Option<String>) -> Result<()> {
        let (jvm, class_loader) = KtfJvmSupport::init(core, system, Some(&jar_filename)).await?;
        let clet_mode = main_class_name.as_deref().is_some_and(is_clet_mode);

        let main_class_name = if let Some(x) = main_class_name {
            x
        } else {
            return Err(WieError::FatalError("Main class not found".into()));
        };

        let main_class_name = main_class_name.replace('.', "/");

        let main_class_name_java = JavaLangString::from_rust_string(&jvm, &main_class_name).await.unwrap();
        let _main_class: Box<dyn ClassInstance> = jvm
            .invoke_virtual(
                &class_loader,
                "net/wie/KtfClassLoader",
                "loadClass",
                "(Ljava/lang/String;)Ljava/lang/Class;",
                (main_class_name_java.clone(),),
            )
            .await
            .unwrap();

        KtfJvmSupport::register_static_classes(core, &jvm, class_loader, &main_class_name).await?;

        let mut args_array = jvm.instantiate_array("Ljava/lang/String;", 1).await.unwrap();
        jvm.store_array(&mut args_array, 0, vec![main_class_name_java]).await.unwrap();
        let result: JvmResult<()> = jvm
            .invoke_static("org/kwis/msp/lcdui/Main", "main", "([Ljava/lang/String;)V", (args_array,))
            .await;

        let result = match result {
            Ok(()) if clet_mode => KtfJvmSupport::disable_midp_paint(&jvm).await,
            result => result,
        };

        JvmSupport::finish_launch(&jvm, result).await
    }
}

// Tasks and SVC handlers hold clones of the system, the core and the JVM, so dropping the emulator frees
// nothing unless the cycles are cut.
impl Drop for KtfEmulator {
    fn drop(&mut self) {
        self.system.teardown();
        self.core.teardown();
        wie_jvm_support::guest_roots::forget(self.core.id());
    }
}

impl Emulator for KtfEmulator {
    fn handle_event(&mut self, event: Event) {
        self.system.event_queue().push(event)
    }

    fn take_pacing(&mut self) -> Pacing {
        self.system.pacing().take()
    }

    fn tick_for(&mut self, budget_ms: u64) -> Result<()> {
        self.system.tick_for(budget_ms).map_err(|x| {
            let reg_stack = self.core.dump_reg_stack(IMAGE_BASE);
            match x {
                WieError::FatalError(msg) => WieError::FatalError(format!("{msg}\n{reg_stack}")),
                _ => WieError::FatalError(format!("{x}\n{reg_stack}")),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc};
    use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

    use test_utils::TestPlatform;
    use wie_backend::{System, YieldFuture};
    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::{Result, WieError};

    use super::{KtfJvmSupport, KtfTaskRunner, drop_device_bound_saves, private_path};

    #[test]
    fn private_directory_mounts_at_the_root_in_either_case() {
        assert_eq!(private_path("P/res/save.sav"), "res/save.sav");
        assert_eq!(private_path("p/imgChar_.dat"), "imgChar_.dat");
        assert_eq!(private_path("res/other.sav"), "res/other.sav");
    }

    #[test]
    fn device_bound_saves_dropped_for_listed_jar_only() {
        use alloc::{collections::BTreeMap, string::ToString, vec};

        let archive = |names: &[&str]| names.iter().map(|name| (name.to_string(), vec![1])).collect::<BTreeMap<_, _>>();
        let table: &[([u8; 16], &[&str])] = &[(md5::compute(b"listed").0, &["res/save.sav", "res/savem.sav"])];

        let mut files = archive(&["A.jar", "__adf__", "P/res/save.sav", "res/savem.sav", "P/res/other.sav"]);
        drop_device_bound_saves(&mut files, b"listed", table);
        assert_eq!(files.keys().collect::<alloc::vec::Vec<_>>(), ["A.jar", "P/res/other.sav", "__adf__"]);

        // A lowercase private directory is mounted at the same path, so it is dropped the same way.
        let mut files = archive(&["A.jar", "p/res/save.sav", "p/res/other.sav"]);
        drop_device_bound_saves(&mut files, b"listed", table);
        assert_eq!(files.keys().collect::<alloc::vec::Vec<_>>(), ["A.jar", "p/res/other.sav"]);

        // Any other jar — every other title — mounts exactly what it shipped.
        let mut files = archive(&["A.jar", "P/res/save.sav", "P/res/savem.sav"]);
        drop_device_bound_saves(&mut files, b"other", table);
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn jar_filename_falls_back_to_the_only_jar() {
        use alloc::{collections::BTreeMap, string::ToString, vec::Vec};

        let files = |names: &[&str]| names.iter().map(|name| (name.to_string(), Vec::new())).collect::<BTreeMap<_, _>>();

        assert_eq!(super::jar_filename("A", &files(&["__adf__", "A.jar", "B.jar"])), "A.jar");
        assert_eq!(super::jar_filename("A", &files(&["__adf__", "B.jar"])), "B.jar");
        assert_eq!(super::jar_filename("A", &files(&["__adf__", "B.jar", "C.jar"])), "A.jar");
        assert_eq!(super::jar_filename("A", &files(&["__adf__"])), "A.jar");
    }

    #[test]
    fn clet_mode_is_selected_from_adf_mclass() {
        assert!(super::is_clet_mode("Clet"));
        assert!(!super::is_clet_mode("MIDlet"));
        assert!(!super::is_clet_mode(""));
    }

    #[test]
    fn switches_jvm_thread_context_between_tasks() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;

        let mut system = System::new(Box::new(TestPlatform::new()), "", "", KtfTaskRunner { core: core.clone() });
        let contexts = Arc::new([AtomicU32::new(0), AtomicU32::new(0)]);
        let completed = Arc::new(AtomicUsize::new(0));

        for index in 0..2 {
            let core = core.clone();
            let contexts = contexts.clone();
            let completed = completed.clone();
            system.spawn(async move || {
                let before = KtfJvmSupport::current_thread_context(&core)?;
                YieldFuture::new().await;
                let after = KtfJvmSupport::current_thread_context(&core)?;
                if before != after {
                    return Err(WieError::FatalError("KTF JVM thread context changed while the task was suspended".into()));
                }

                contexts[index].store(before, Ordering::Relaxed);
                completed.fetch_add(1, Ordering::Relaxed);

                Ok(())
            });
        }

        while completed.load(Ordering::Relaxed) != 2 {
            system.tick()?;
        }

        let first = contexts[0].load(Ordering::Relaxed);
        let second = contexts[1].load(Ordering::Relaxed);
        assert_ne!(first, 0);
        assert_ne!(second, 0);
        assert_ne!(first, second);

        Ok(())
    }
}
