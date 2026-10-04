//! KTF's side of the guest root scan (`wie_jvm_support::guest_roots`): the words it reads — the
//! core's registers and stacks; the image is registered as a region by `load_native` — and how it
//! decodes an instance.

use alloc::{boxed::Box, vec::Vec};

use jvm::{ClassInstance, Jvm};

use wie_core_arm::ArmCore;
use wie_jvm_support::{guest_roots, native::NativeJavaValueCodec};
use wie_util::{ByteRead, Result};

use super::value::JavaValueCodec;

#[derive(Clone)]
struct KtfGuest(ArmCore);

impl ByteRead for KtfGuest {
    fn read_bytes(&self, address: u32, result: &mut [u8]) -> Result<usize> {
        self.0.read_bytes(address, result)
    }
}

impl guest_roots::GuestMemory for KtfGuest {
    fn id(&self) -> usize {
        self.0.id()
    }

    fn root_words(&self) -> Result<Vec<u32>> {
        self.0.guest_root_words()
    }

    fn object_from_raw(&self, raw: u32) -> Option<Box<dyn ClassInstance>> {
        JavaValueCodec::new(&self.0).object_from_raw(raw)
    }
}

pub fn install(jvm: &Jvm, core: &ArmCore) {
    guest_roots::install(jvm, KtfGuest(core.clone()));
}
