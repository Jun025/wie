mod bucket;
mod list;

use wie_util::{Result, WieError};

use crate::{
    ArmCore,
    core::{HEAP_BASE, HEAP_SIZE},
};

use self::{
    bucket::{BUCKET_MAX, BucketAllocator},
    list::ListAllocator,
};

pub struct Allocator;

impl Allocator {
    pub fn init(core: &mut ArmCore) -> Result<()> {
        core.map(HEAP_BASE, HEAP_SIZE)?;

        ListAllocator::init(core, HEAP_BASE, HEAP_SIZE / 2)?;
        BucketAllocator::init(core, HEAP_BASE + HEAP_SIZE / 2, HEAP_SIZE / 2)?;

        Ok(())
    }

    pub fn alloc(core: &mut ArmCore, size: u32) -> Result<u32> {
        let result = if size > BUCKET_MAX as _ {
            ListAllocator::alloc(core, HEAP_BASE, HEAP_SIZE / 2, size)
        } else {
            BucketAllocator::alloc(core, HEAP_BASE + HEAP_SIZE / 2, size)
        };
        if let Err(WieError::AllocationFailure) = &result {
            // Which half ran out, and how full each is: «the guest heap is full» says nothing about why.
            tracing::error!(
                "guest heap allocation of {size:#x} bytes failed; list {}; buckets {}",
                ListAllocator::usage(core, HEAP_BASE, HEAP_SIZE / 2).unwrap_or_default(),
                BucketAllocator::usage(core, HEAP_BASE + HEAP_SIZE / 2).unwrap_or_default()
            );
        }
        result
    }

    // Which allocator owns a block is a fact about its address; `size` is only the caller's claim.
    pub fn free(core: &mut ArmCore, address: u32, size: u32) -> Result<()> {
        if address < HEAP_BASE + HEAP_SIZE / 2 {
            ListAllocator::free(core, address)
        } else {
            BucketAllocator::free(core, HEAP_BASE + HEAP_SIZE / 2, address, size)
        }
    }

    pub fn is_allocated(core: &ArmCore, address: u32, size: u32) -> Result<bool> {
        if size > BUCKET_MAX as _ {
            ListAllocator::is_allocated(core, HEAP_BASE, HEAP_SIZE / 2, address, size)
        } else {
            BucketAllocator::is_allocated(core, HEAP_BASE + HEAP_SIZE / 2, address, size)
        }
    }
}

#[cfg(test)]
mod tests {
    use wie_util::Result;

    use crate::{Allocator, ArmCore};

    #[test]
    fn allocation_status_tracks_bucket_and_list_allocations() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;

        let bucket = Allocator::alloc(&mut core, 12)?;
        let list = Allocator::alloc(&mut core, 1024)?;
        assert!(Allocator::is_allocated(&core, bucket, 12)?);
        assert!(Allocator::is_allocated(&core, list, 1024)?);
        assert!(!Allocator::is_allocated(&core, bucket + 4, 12)?);

        Allocator::free(&mut core, bucket, 12)?;
        Allocator::free(&mut core, list, 1024)?;
        assert!(!Allocator::is_allocated(&core, bucket, 12)?);
        assert!(!Allocator::is_allocated(&core, list, 1024)?);

        Ok(())
    }

    // A block goes back to the allocator that owns its address, whatever size the caller claims.
    #[test]
    fn free_follows_the_address_not_the_size() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;

        let list = Allocator::alloc(&mut core, 1024)?;
        Allocator::free(&mut core, list, 4)?;
        assert!(!Allocator::is_allocated(&core, list, 1024)?);

        Ok(())
    }
}
