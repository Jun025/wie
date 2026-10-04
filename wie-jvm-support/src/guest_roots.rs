//! The guest's references as GC roots, for carriers whose guest code is AOT-compiled ARM (LGT, KTF).
//!
//! AOT guest code keeps object pointers in registers, on its stacks and in its data sections, none
//! of which the collector sees. Every live instance's blocks are recorded here, and at collection
//! time [`GuestRoots`] — one global reference installed per JVM — reports as its fields every
//! instance a guest word points into. Conservative: a word that only looks like a pointer keeps
//! an object alive one collection longer, which is the safe direction; missing a real pointer
//! frees an object the guest still uses.
//!
//! The carrier supplies the words ([`GuestMemory::root_words`] plus the regions it registers) and
//! decodes an instance pointer; the ledger, the scan and the stress mode are shared. Every table is
//! keyed by the emulator's core id (`ArmCore::id`).
//!
//! ponytail: the guest's own malloc blocks are not scanned — a pointer kept only inside one is
//! missed; scan the guest heap's allocated blocks if a title is seen losing an object that way.

use alloc::{boxed::Box, collections::BTreeMap, collections::BTreeSet, string::String, vec, vec::Vec};
use core::{
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
};

use jvm::{ClassDefinition, ClassInstance, ClassInstanceRef, Field, GlobalRef, JavaValue, Jvm, Method, Result as JvmResult};
use jvm_types::{ClassAccessFlags, FieldAccessFlags};
use spin::Mutex;

use wie_util::{ByteRead, ByteWrite, Result};

/// What the scan needs from a carrier's guest. One value per emulator, cheap to clone.
pub trait GuestMemory: ByteRead + Clone + Send + Sync + 'static {
    /// The key of this emulator's entries: `ArmCore::id`.
    fn id(&self) -> usize;
    /// Every word guest code may hold a pointer in, besides the regions registered with
    /// [`add_region`]: registers, stacks, and whatever ledger the carrier keeps in guest memory.
    fn root_words(&self) -> Result<Vec<u32>>;
    /// The instance whose header is at `raw` (a tracked instance pointer).
    fn object_from_raw(&self, raw: u32) -> Option<Box<dyn ClassInstance>>;
}

// (core id, block start) → (block end, inclusive, so a one-past-the-end pointer still counts; the
// instance pointer). Two blocks per instance: its header and its field storage — AOT code may
// keep only the storage pointer (or an element address inside it) across a call.
static BLOCKS: Mutex<BTreeMap<(usize, u32), (u32, u32)>> = Mutex::new(BTreeMap::new());
// core id → guest memory ranges to scan besides registers and stacks (writable image sections).
static REGIONS: Mutex<BTreeMap<usize, Vec<(u32, u32)>>> = Mutex::new(BTreeMap::new());
// core id → the scan's global reference, for cores whose JVM has finished bootstrapping and has the
// scan installed. A collection before that is unsafe for its own reason: bootstrap classes have no
// java/lang/Class object yet. Held here, not forgotten: the reference holds the core, and a
// forgotten one kept every booted core alive.
static INSTALLED: Mutex<BTreeMap<usize, GlobalRef<()>>> = Mutex::new(BTreeMap::new());
// A core's entries leave all three in `forget`, which the emulator calls as it is dropped: a freed
// core's address can be the next core's id, and a reused id would inherit them.

pub fn track(id: usize, ptr_raw: u32, header_size: usize, ptr_fields: u32, storage_size: usize) {
    let mut blocks = BLOCKS.lock();
    blocks.insert((id, ptr_raw), (ptr_raw + header_size as u32, ptr_raw));
    blocks.insert((id, ptr_fields), (ptr_fields + storage_size.max(size_of::<u32>()) as u32, ptr_raw));
}

pub fn untrack(id: usize, ptr_raw: u32, ptr_fields: u32) {
    let mut blocks = BLOCKS.lock();
    for start in [ptr_raw, ptr_fields] {
        if blocks.get(&(id, start)).is_some_and(|(_, owner)| *owner == ptr_raw) {
            blocks.remove(&(id, start));
        }
    }
}

pub fn is_tracked(id: usize, ptr_raw: u32) -> bool {
    BLOCKS.lock().get(&(id, ptr_raw)).is_some_and(|(_, owner)| *owner == ptr_raw)
}

pub fn add_region(id: usize, address: u32, size: u32) {
    REGIONS.lock().entry(id).or_default().push((address, size));
}

pub fn forget(id: usize) {
    BLOCKS.lock().retain(|(core_id, _), _| *core_id != id);
    REGIONS.lock().remove(&id);
    let roots = INSTALLED.lock().remove(&id);
    drop(roots);
}

pub fn install<M: GuestMemory>(jvm: &Jvm, memory: M) {
    let id = memory.id();
    let roots = ClassInstanceRef::<()>::new(Some(Box::new(GuestRoots { memory })));
    // Released by `forget` only: the scan must run at every collection for as long as this JVM lives.
    if let Some(roots) = jvm.new_global_ref(&roots) {
        INSTALLED.lock().insert(id, roots);
    }
}

/// Every tracked instance a guest word points into.
fn roots<M: GuestMemory>(memory: &M) -> BTreeSet<u32> {
    let id = memory.id();
    let words = (|| -> Result<Vec<u32>> {
        let mut words = memory.root_words()?;
        for (address, size) in REGIONS.lock().get(&id).cloned().unwrap_or_default() {
            let mut region = vec![0; size as usize & !3];
            memory.read_bytes(address, &mut region)?;
            words.extend(region.as_chunks::<4>().0.iter().map(|word| u32::from_le_bytes(*word)));
        }
        Ok(words)
    })();

    let blocks = BLOCKS.lock();
    let words = match words {
        Ok(words) => words,
        Err(error) => {
            // Collecting with what we could not see would free what the guest still uses.
            tracing::error!("guest root scan failed, keeping every guest object: {error}");
            return blocks.range((id, 0)..=(id, u32::MAX)).map(|(_, (_, owner))| *owner).collect();
        }
    };

    // Blocks do not overlap, so the last one by start ends highest: a word outside [low, high]
    // points into none of them.
    let ours = || blocks.range((id, 0)..=(id, u32::MAX));
    let (Some(((_, low), _)), Some((_, (high, _)))) = (ours().next(), ours().next_back()) else {
        return BTreeSet::new();
    };
    let (low, high) = (*low, *high);

    let mut roots = BTreeSet::new();
    for word in words.into_iter().filter(|word| (low..=high).contains(word)) {
        // Two candidates: the block the word is in, and the one before it when the word is its end.
        for (_, (end, owner)) in blocks.range((id, 0)..=(id, word)).rev().take(2) {
            if word <= *end {
                roots.insert(*owner);
            }
        }
    }
    // The live-object curve a leak shows up in: instances at this collection, before it frees
    // anything (RUST_LOG=wie_jvm_support::guest_roots=debug).
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

pub fn stress_collect(jvm: &Jvm, id: usize) {
    let every = STRESS_EVERY.load(Ordering::Relaxed);
    if every != 0 && INSTALLED.lock().contains_key(&id) && STRESS_CALLS.fetch_add(1, Ordering::Relaxed).is_multiple_of(every as u64) {
        STRESS_COLLECTIONS.fetch_add(1, Ordering::Relaxed);
        if let Err(error) = jvm.collect_garbage() {
            tracing::error!("gc stress collection failed: {error:?}");
        }
    }
}

pub fn stress_poison(memory: &mut impl ByteWrite, address: u32, size: usize) {
    if STRESS_EVERY.load(Ordering::Relaxed) != 0 {
        let _ = memory.write_bytes(address, &POISON.to_le_bytes().repeat(size.div_ceil(4)));
    }
}

/// Called with every reference the guest hands the host: in stress mode, one that is not a live
/// instance is a reference the collector freed under the guest.
pub fn stress_check(id: usize, ptr_raw: u32) {
    if ptr_raw != 0 && STRESS_EVERY.load(Ordering::Relaxed) != 0 && !is_tracked(id, ptr_raw) {
        STRESS_DANGLING.fetch_add(1, Ordering::Relaxed);
        tracing::error!("gc stress: guest reference {ptr_raw:#x} is not a live instance");
    }
}

#[derive(Clone)]
struct GuestRoots<M> {
    memory: M,
}

#[async_trait::async_trait]
impl<M: GuestMemory> ClassInstance for GuestRoots<M> {
    fn destroy(self: Box<Self>) {}

    fn identity(&self) -> usize {
        usize::MAX
    }

    fn shallow_clone(&self) -> JvmResult<Box<dyn ClassInstance>> {
        Ok(Box::new(self.clone()))
    }

    fn class_definition(&self) -> Box<dyn ClassDefinition> {
        Box::new(GuestRootsClass { memory: self.memory.clone() })
    }

    fn equals(&self, other: &dyn ClassInstance) -> JvmResult<bool> {
        Ok(other.as_any().is::<GuestRoots<M>>())
    }

    fn get_field(&self, field: &dyn Field) -> JvmResult<JavaValue> {
        let root = field.as_any().downcast_ref::<GuestRoot>().map(|root| root.0);
        Ok(JavaValue::Object(root.and_then(|ptr| self.memory.object_from_raw(ptr))))
    }

    fn put_field(&mut self, _: &dyn Field, _: JavaValue) -> JvmResult<()> {
        Ok(())
    }
}

impl<M> Hash for GuestRoots<M> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        usize::MAX.hash(state);
    }
}

impl<M> Debug for GuestRoots<M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "GuestRoots")
    }
}

#[derive(Clone)]
struct GuestRootsClass<M> {
    memory: M,
}

#[async_trait::async_trait]
impl<M: GuestMemory> ClassDefinition for GuestRootsClass<M> {
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
        roots(&self.memory)
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

impl<M> Debug for GuestRootsClass<M> {
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

#[cfg(test)]
mod tests {
    use super::{REGIONS, add_region, forget, is_tracked, track};

    // A freed core's id can be the next core's: `forget` must leave nothing under it.
    #[test]
    fn forget_drops_every_entry_of_the_core() {
        let id = usize::MAX - 1;
        track(id, 0x4000_0000, 12, 0x4000_0100, 8);
        add_region(id, 0x1000, 0x10);
        forget(id);
        assert!(!is_tracked(id, 0x4000_0000));
        assert!(!REGIONS.lock().contains_key(&id));
    }
}
