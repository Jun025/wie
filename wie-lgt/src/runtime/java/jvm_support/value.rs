use alloc::boxed::Box;
use core::sync::atomic::{AtomicU32, Ordering};

use jvm::ClassInstance;

use wie_core_arm::ArmCore;
use wie_jvm_support::native::NativeJavaValueCodec;

use super::{JavaArrayClassInstance, JavaClassInstance};

#[derive(Clone)]
pub struct JavaValueCodec {
    core: ArmCore,
}

impl JavaValueCodec {
    pub fn new(core: &ArmCore) -> Self {
        Self { core: core.clone() }
    }
}

impl NativeJavaValueCodec for JavaValueCodec {
    fn object_from_raw(&self, raw: u32) -> Option<Box<dyn ClassInstance>> {
        let instance = JavaClassInstance::from_raw(raw, &self.core);
        // Reading the class is also the liveness check: it dereferences `raw` and then the
        // instance's `ptr_dispatch_table`, which every live instance has set.
        let class = match instance.class() {
            Ok(class) => class,
            Err(error) => {
                tracing::warn!("LGT object reference {raw:#x} does not point at a live instance: {error}");
                return None;
            }
        };
        // A readable dispatch table only proves the instance header is live; the class word it
        // holds can still point nowhere (measured: a word of `0x104c02b4` panicked the host inside
        // the infallible `ClassDefinition::name`). This function already answers null for an
        // object it cannot read, so an unreadable class gets the same answer.
        let name = match class.try_name() {
            Ok(name) => name,
            Err(error) => {
                report_unreadable_class(raw, class.ptr_raw, &error);
                return None;
            }
        };
        if name.starts_with('[') {
            Some(Box::new(JavaArrayClassInstance::from_raw(raw, &self.core)))
        } else {
            Some(Box::new(instance))
        }
    }

    fn object_to_raw(&self, object: &dyn ClassInstance) -> u32 {
        if let Some(instance) = object.as_any().downcast_ref::<JavaClassInstance>() {
            instance.ptr_raw
        } else {
            object.as_any().downcast_ref::<JavaArrayClassInstance>().unwrap().class_instance.ptr_raw
        }
    }
}

/// How many references `object_from_raw` has folded to null because their class was unreadable.
/// Only the first is a `warn!` — a title that hits this tends to hit it every tick, and on the web
/// build every warning is a console line (the same trade `java_is_class_assignable` made).
/// ponytail: process-wide, not per-title — one wasm instance / validator run is one title.
static UNREADABLE_CLASS_REFERENCES: AtomicU32 = AtomicU32::new(0);

fn report_unreadable_class(raw: u32, ptr_class: u32, error: &wie_util::WieError) {
    if UNREADABLE_CLASS_REFERENCES.fetch_add(1, Ordering::Relaxed) == 0 {
        tracing::warn!(
            "LGT object reference {raw:#x} has an unreadable class {ptr_class:#x}: {error} — decoding as null (further occurrences at trace level)"
        );
    } else {
        tracing::trace!("LGT object reference {raw:#x} has an unreadable class {ptr_class:#x}: {error} — decoding as null");
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use core::{
        mem::{offset_of, size_of},
        sync::atomic::Ordering,
    };

    use jvm::{JavaType, JavaValue};
    use wipi_types::lgt::java::{
        LgtJavaClass as RawJavaClass, LgtJavaClassDescriptor as RawJavaClassDescriptor, LgtJavaClassInstance as RawJavaClassInstance,
    };

    use wie_core_arm::{Allocator, ArmCore};
    use wie_jvm_support::native::NativeJavaValueCodec;
    use wie_util::{ByteWrite, Result, read_generic, write_generic, write_null_terminated_string_bytes};

    use super::{JavaValueCodec, UNREADABLE_CLASS_REFERENCES};

    /// Writes a minimal live instance and returns its pointer.
    fn live_instance(core: &mut ArmCore, name: &[u8]) -> Result<u32> {
        let ptr_name = Allocator::alloc(core, name.len() as u32 + 1)?;
        write_null_terminated_string_bytes(core, ptr_name, name)?;

        let ptr_descriptor = Allocator::alloc(core, size_of::<RawJavaClassDescriptor>() as u32)?;
        core.write_bytes(ptr_descriptor, &[0; size_of::<RawJavaClassDescriptor>()])?;
        write_generic(core, ptr_descriptor + offset_of!(RawJavaClassDescriptor, ptr_name) as u32, ptr_name)?;

        let ptr_class = Allocator::alloc(core, size_of::<RawJavaClass>() as u32)?;
        core.write_bytes(ptr_class, &[0; size_of::<RawJavaClass>()])?;
        write_generic(core, ptr_class + offset_of!(RawJavaClass, ptr_descriptor) as u32, ptr_descriptor)?;

        // The dispatch table holds the class pointer in word zero, which is what `class()` reads.
        let ptr_dispatch_table = Allocator::alloc(core, size_of::<u32>() as u32)?;
        write_generic(core, ptr_dispatch_table, ptr_class)?;

        let ptr_fields = Allocator::alloc(core, size_of::<u32>() as u32)?;
        write_generic(core, ptr_fields, 0u32)?;

        let ptr_instance = Allocator::alloc(core, size_of::<RawJavaClassInstance>() as u32)?;
        write_generic(
            core,
            ptr_instance,
            RawJavaClassInstance {
                ptr_dispatch_table,
                unk1: 0,
                ptr_fields,
            },
        )?;
        Ok(ptr_instance)
    }

    /// A reference-typed word whose instance header has no dispatch table is not a live object.
    /// Decoding it must read as null rather than panic — two titles panicked the host here, and
    /// the `jvm` traits above this frame have nowhere to put the error (see the trait's doc).
    /// The live half is asserted too, so "always null" is not a passing shortcut.
    #[test]
    fn a_reference_to_a_dead_instance_decodes_to_null() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;
        let codec = JavaValueCodec::new(&core);
        let r#type = JavaType::Class(String::from("java/lang/Object"));

        let live = live_instance(&mut core, b"java/lang/Object")?;
        assert!(matches!(codec.decode_word(live, &r#type), JavaValue::Object(Some(_))));

        // Same instance, dispatch table cleared: exactly the LGT shape that panicked.
        write_generic(&mut core, live + offset_of!(RawJavaClassInstance, ptr_dispatch_table) as u32, 0u32)?;
        assert!(matches!(codec.decode_word(live, &r#type), JavaValue::Object(None)));

        // And a word that is not even an address still reads as null, not a panic.
        assert!(matches!(codec.decode_word(0xdead_beef, &r#type), JavaValue::Object(None)));

        Ok(())
    }

    /// A live instance whose dispatch table holds an unreadable class word must decode to null.
    ///
    /// This is the half the test above does not reach: `class()` succeeds here — the header and
    /// the dispatch table both read — and it is the class word *inside* the table that points
    /// nowhere. `0x104c02b4` is the word 놈3 produced (2026-09-23). Before this, `object_from_raw`
    /// called the infallible `ClassDefinition::name` on it and the host panicked at
    /// `class_definition.rs` `try_name().unwrap()` with `InvalidMemoryAccess(0x104c02b4)`.
    /// Restoring that call turns this test into that panic.
    #[test]
    fn a_reference_whose_class_word_is_unreadable_decodes_to_null() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;
        let codec = JavaValueCodec::new(&core);
        let r#type = JavaType::Class(String::from("java/lang/Object"));

        let live = live_instance(&mut core, b"java/lang/Object")?;
        assert!(matches!(codec.decode_word(live, &r#type), JavaValue::Object(Some(_))));

        let before = UNREADABLE_CLASS_REFERENCES.load(Ordering::Relaxed);
        let instance: RawJavaClassInstance = read_generic(&core, live)?;
        write_generic(&mut core, instance.ptr_dispatch_table, 0x104c_02b4u32)?;
        assert!(matches!(codec.decode_word(live, &r#type), JavaValue::Object(None)));
        assert!(matches!(codec.decode_word(live, &r#type), JavaValue::Object(None)));
        // Both occurrences are counted; only the one that moves the counter off zero warns.
        assert_eq!(UNREADABLE_CLASS_REFERENCES.load(Ordering::Relaxed) - before, 2);

        Ok(())
    }
}
