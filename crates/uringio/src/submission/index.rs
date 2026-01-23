use std::{
    io::Result,
    ops::{Deref, DerefMut},
    slice::from_raw_parts_mut,
};

use crate::{
    platform::{
        iouring::{IoUringParams, IoUringSetupFlags},
        mmap::Mmap,
    },
    shared::error::err,
};

/// ## Submission Index
#[derive(Debug, Default)]
pub struct SubmissionIndex<'fd> {
    indices: &'fd mut [u32],
}

#[inline]
const fn submission_indices<'fd>(sq_mmap: &Mmap, params: &IoUringParams) -> &'fd mut [u32] {
    unsafe {
        let IoUringParams { sq_off, .. } = params;
        let head = sq_mmap.offset(sq_off.array).cast::<u32>();
        let size = sq_mmap.offset(sq_off.ring_entries).cast::<u32>().read();
        from_raw_parts_mut(head.as_ptr(), size as usize)
    }
}

impl SubmissionIndex<'_> {
    pub fn new(sq_mmap: &Mmap, params: &IoUringParams) -> Result<Self> {
        if params.flags.contains(IoUringSetupFlags::NO_SQARRAY) {
            return err!("IoUring setup with IORING_SETUP_NO_SQARRAY flag");
        }

        Ok(Self { indices: submission_indices(sq_mmap, params) })
    }

    pub fn setup(sq_mmap: &Mmap, params: &IoUringParams) {
        if params.flags.contains(IoUringSetupFlags::NO_SQARRAY) {
            return;
        }

        let indices = submission_indices(sq_mmap, params);

        for (idx, item) in indices.iter_mut().enumerate() {
            *item = idx as u32;
        }
    }
}

impl Deref for SubmissionIndex<'_> {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        self.indices
    }
}

impl DerefMut for SubmissionIndex<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.indices
    }
}
