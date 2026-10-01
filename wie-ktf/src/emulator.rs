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

/// Remove the saves `table` names for this jar, both bare and under `P/` (`load` mounts both
/// spellings at the same path).
fn drop_device_bound_saves(files: &mut BTreeMap<String, Vec<u8>>, jar: &[u8], table: &[([u8; 16], &[&str])]) {
    let hash = md5::compute(jar).0;
    for (_, names) in table.iter().filter(|(md5, _)| *md5 == hash) {
        for name in *names {
            for key in [(*name).to_owned(), format!("P/{name}")] {
                if files.remove(&key).is_some() {
                    tracing::info!("Not mounting device-bound save {key}");
                }
            }
        }
    }
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
            let path = path.trim_start_matches("P/");
            system.filesystem().add_virtual(path, data.clone());
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

        if let Err(x) = result {
            return Err(JvmSupport::to_wie_err(&jvm, x).await);
        }

        if clet_mode && let Err(x) = KtfJvmSupport::disable_midp_paint(&jvm).await {
            return Err(JvmSupport::to_wie_err(&jvm, x).await);
        }

        Ok(())
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

    use super::{KtfJvmSupport, KtfTaskRunner, drop_device_bound_saves};

    #[test]
    fn device_bound_saves_dropped_for_listed_jar_only() {
        use alloc::{collections::BTreeMap, string::ToString, vec};

        let archive = |names: &[&str]| names.iter().map(|name| (name.to_string(), vec![1])).collect::<BTreeMap<_, _>>();
        let table: &[([u8; 16], &[&str])] = &[(md5::compute(b"listed").0, &["res/save.sav", "res/savem.sav"])];

        let mut files = archive(&["A.jar", "__adf__", "P/res/save.sav", "res/savem.sav", "P/res/other.sav"]);
        drop_device_bound_saves(&mut files, b"listed", table);
        assert_eq!(files.keys().collect::<alloc::vec::Vec<_>>(), ["A.jar", "P/res/other.sav", "__adf__"]);

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
