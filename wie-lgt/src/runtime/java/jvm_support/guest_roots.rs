//! The guest's references as GC roots.
//!
//! AOT guest code keeps object pointers in registers, on its stacks and in its data sections, none
//! of which the collector sees. Every live instance's blocks are recorded here, and at collection
//! time [`GuestRoots`] — one global reference installed per JVM — reports as its fields every
//! instance a guest word points into. Conservative: a word that only looks like a pointer keeps
//! an object alive one collection longer, which is the safe direction; missing a real pointer
//! frees an object the guest still uses.

use alloc::{boxed::Box, collections::BTreeMap, collections::BTreeSet, string::String, vec, vec::Vec};
use core::{
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    mem::size_of,
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
};

use jvm::{ClassDefinition, ClassInstance, ClassInstanceRef, Field, JavaValue, Jvm, Method, Result as JvmResult};
use jvm_types::{ClassAccessFlags, FieldAccessFlags};
use spin::Mutex;
use wipi_types::lgt::java::LgtJavaClassInstance as RawJavaClassInstance;

use wie_core_arm::ArmCore;
use wie_jvm_support::native::NativeJavaValueCodec;
use wie_util::{ByteRead, ByteWrite, Result};

use crate::runtime::java::exception;

use super::value::JavaValueCodec;

// (core id, block start) → (block end, inclusive, so a one-past-the-end pointer still counts; the
// instance pointer). Two blocks per instance: its header and its field storage — AOT code may
// keep only the storage pointer (or an element address inside it) across a call.
static BLOCKS: Mutex<BTreeMap<(usize, u32), (u32, u32)>> = Mutex::new(BTreeMap::new());
// core id → guest memory ranges to scan besides registers and stacks (writable image sections).
static REGIONS: Mutex<BTreeMap<usize, Vec<(u32, u32)>>> = Mutex::new(BTreeMap::new());
// core ids whose JVM has finished bootstrapping and has the scan installed. A collection before that
// is unsafe for its own reason: bootstrap classes have no java/lang/Class object yet.
static INSTALLED: Mutex<BTreeSet<usize>> = Mutex::new(BTreeSet::new());

const HEAP: core::ops::Range<u32> = 0x4000_0000..0x5000_0000;

pub fn track(core: &ArmCore, ptr_raw: u32, ptr_fields: u32, storage_size: usize) {
    let mut blocks = BLOCKS.lock();
    blocks.insert((core.id(), ptr_raw), (ptr_raw + size_of::<RawJavaClassInstance>() as u32, ptr_raw));
    blocks.insert((core.id(), ptr_fields), (ptr_fields + storage_size.max(size_of::<u32>()) as u32, ptr_raw));
}

pub fn untrack(core: &ArmCore, ptr_raw: u32, ptr_fields: u32) {
    let mut blocks = BLOCKS.lock();
    for start in [ptr_raw, ptr_fields] {
        if blocks.get(&(core.id(), start)).is_some_and(|(_, owner)| *owner == ptr_raw) {
            blocks.remove(&(core.id(), start));
        }
    }
}

pub fn is_tracked(core: &ArmCore, ptr_raw: u32) -> bool {
    BLOCKS.lock().get(&(core.id(), ptr_raw)).is_some_and(|(_, owner)| *owner == ptr_raw)
}

pub fn add_region(core: &ArmCore, address: u32, size: u32) {
    REGIONS.lock().entry(core.id()).or_default().push((address, size));
}

pub fn install(jvm: &Jvm, core: &ArmCore) {
    INSTALLED.lock().insert(core.id());
    let roots = ClassInstanceRef::<()>::new(Some(Box::new(GuestRoots { core: core.clone() })));
    // Never released: the scan must run at every collection for as long as this JVM lives.
    core::mem::forget(jvm.new_global_ref(&roots));
}

/// Every tracked instance a guest word points into.
fn roots(core: &ArmCore) -> BTreeSet<u32> {
    let words = (|| -> Result<Vec<u32>> {
        let mut words = core.guest_root_words()?;
        words.extend(exception::root_words(core)?);
        for (address, size) in REGIONS.lock().get(&core.id()).cloned().unwrap_or_default() {
            let mut region = vec![0; size as usize & !3];
            core.read_bytes(address, &mut region)?;
            words.extend(region.as_chunks::<4>().0.iter().map(|word| u32::from_le_bytes(*word)));
        }
        Ok(words)
    })();

    let blocks = BLOCKS.lock();
    let id = core.id();
    let words = match words {
        Ok(words) => words,
        Err(error) => {
            // Collecting with what we could not see would free what the guest still uses.
            tracing::error!("guest root scan failed, keeping every guest object: {error}");
            return blocks.range((id, 0)..=(id, u32::MAX)).map(|(_, (_, owner))| *owner).collect();
        }
    };

    let mut roots = BTreeSet::new();
    for word in words.into_iter().filter(|word| HEAP.contains(word)) {
        // Two candidates: the block the word is in, and the one before it when the word is its end.
        for (_, (end, owner)) in blocks.range((id, 0)..=(id, word)).rev().take(2) {
            if word <= *end {
                roots.insert(*owner);
            }
        }
    }
    // The live-object curve a leak shows up in: instances at this collection, before it frees
    // anything (RUST_LOG=wie_lgt::runtime::java::jvm_support::guest_roots=debug).
    if tracing::enabled!(tracing::Level::DEBUG) {
        let instances = blocks
            .range((id, 0)..=(id, u32::MAX))
            .filter(|(start, (_, owner))| start.1 == *owner)
            .count();
        tracing::debug!("guest roots {} of {instances} instances", roots.len());
    }
    roots
}

// GC stress mode (wie_validate --gc-stress N): collect at every Nth guest→host call and poison what
// a collection frees, so a missed root shows up as a fault at 0xdeaddead or as a dangling count.
static STRESS_EVERY: AtomicU32 = AtomicU32::new(0);
static STRESS_CALLS: AtomicU64 = AtomicU64::new(0);
static STRESS_COLLECTIONS: AtomicU64 = AtomicU64::new(0);
static STRESS_DANGLING: AtomicU64 = AtomicU64::new(0);
pub const POISON: u32 = 0xdead_dead;

pub fn set_gc_stress(every: u32) {
    STRESS_EVERY.store(every, Ordering::Relaxed);
}

/// (collections it forced, guest references to freed instances it saw)
pub fn gc_stress_counts() -> (u64, u64) {
    (STRESS_COLLECTIONS.load(Ordering::Relaxed), STRESS_DANGLING.load(Ordering::Relaxed))
}

pub fn stress_collect(jvm: &Jvm, core: &ArmCore) {
    let every = STRESS_EVERY.load(Ordering::Relaxed);
    if every != 0 && INSTALLED.lock().contains(&core.id()) && STRESS_CALLS.fetch_add(1, Ordering::Relaxed).is_multiple_of(every as u64) {
        STRESS_COLLECTIONS.fetch_add(1, Ordering::Relaxed);
        if let Err(error) = jvm.collect_garbage() {
            tracing::error!("gc stress collection failed: {error:?}");
        }
    }
}

pub fn stress_poison(core: &mut ArmCore, address: u32, size: usize) {
    if STRESS_EVERY.load(Ordering::Relaxed) != 0 {
        let _ = core.write_bytes(address, &POISON.to_le_bytes().repeat(size.div_ceil(4)));
    }
}

/// Called with every reference the guest hands the host: in stress mode, one that is not a live
/// instance is a reference the collector freed under the guest.
pub fn stress_check(core: &ArmCore, ptr_raw: u32) {
    if ptr_raw != 0 && STRESS_EVERY.load(Ordering::Relaxed) != 0 && !is_tracked(core, ptr_raw) {
        STRESS_DANGLING.fetch_add(1, Ordering::Relaxed);
        tracing::error!("gc stress: guest reference {ptr_raw:#x} is not a live instance");
    }
}

#[derive(Clone)]
struct GuestRoots {
    core: ArmCore,
}

#[async_trait::async_trait]
impl ClassInstance for GuestRoots {
    fn destroy(self: Box<Self>) {}

    fn identity(&self) -> usize {
        usize::MAX
    }

    fn shallow_clone(&self) -> JvmResult<Box<dyn ClassInstance>> {
        Ok(Box::new(self.clone()))
    }

    fn class_definition(&self) -> Box<dyn ClassDefinition> {
        Box::new(GuestRootsClass { core: self.core.clone() })
    }

    fn equals(&self, other: &dyn ClassInstance) -> JvmResult<bool> {
        Ok(other.as_any().is::<GuestRoots>())
    }

    fn get_field(&self, field: &dyn Field) -> JvmResult<JavaValue> {
        let root = field.as_any().downcast_ref::<GuestRoot>().map(|root| root.0);
        Ok(JavaValue::Object(
            root.and_then(|ptr| JavaValueCodec::new(&self.core).object_from_raw(ptr)),
        ))
    }

    fn put_field(&mut self, _: &dyn Field, _: JavaValue) -> JvmResult<()> {
        Ok(())
    }
}

impl Hash for GuestRoots {
    fn hash<H: Hasher>(&self, state: &mut H) {
        usize::MAX.hash(state);
    }
}

impl Debug for GuestRoots {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "GuestRoots")
    }
}

#[derive(Clone)]
struct GuestRootsClass {
    core: ArmCore,
}

#[async_trait::async_trait]
impl ClassDefinition for GuestRootsClass {
    fn name(&self) -> String {
        "net/wie/GuestRoots".into()
    }

    fn super_class_name(&self) -> Option<String> {
        None
    }

    fn interface_names(&self) -> Vec<String> {
        Vec::new()
    }

    fn access_flags(&self) -> ClassAccessFlags {
        ClassAccessFlags::empty()
    }

    async fn instantiate(&self, jvm: &Jvm) -> JvmResult<Box<dyn ClassInstance>> {
        Err(jvm.exception("java/lang/InstantiationError", "net/wie/GuestRoots").await)
    }

    async fn prepare(&self, _: &Jvm) -> JvmResult<()> {
        Ok(())
    }

    fn method(&self, _: &str, _: &str, _: bool) -> Option<Box<dyn Method>> {
        None
    }

    fn field(&self, _: &str, _: &str, _: bool) -> Option<Box<dyn Field>> {
        None
    }

    // The scan itself: the collector asks for this class's fields once per collection.
    fn fields(&self) -> Vec<Box<dyn Field>> {
        roots(&self.core)
            .into_iter()
            .map(|ptr| Box::new(GuestRoot(ptr)) as Box<dyn Field>)
            .collect()
    }

    fn get_static_field(&self, _: &dyn Field) -> JvmResult<JavaValue> {
        Ok(JavaValue::Object(None))
    }

    fn put_static_field(&mut self, _: &dyn Field, _: JavaValue) -> JvmResult<()> {
        Ok(())
    }
}

impl Debug for GuestRootsClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "GuestRootsClass")
    }
}

#[derive(Debug)]
struct GuestRoot(u32);

impl Field for GuestRoot {
    fn name(&self) -> String {
        alloc::format!("<guest-root-{:#x}>", self.0)
    }

    fn descriptor(&self) -> String {
        "Ljava/lang/Object;".into()
    }

    fn access_flags(&self) -> FieldAccessFlags {
        FieldAccessFlags::PRIVATE
    }
}
