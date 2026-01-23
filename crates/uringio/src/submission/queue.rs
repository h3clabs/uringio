use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr::NonNull,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{
    platform::{
        iouring::{IoUringParams, IoUringSqFlags},
        mmap::Mmap,
    },
    submission::{index::SubmissionIndex, submitter::Submitter},
    uring::mode::Mode,
};

/// ## Submission Queue
#[derive(Debug)]
pub struct SubmissionQueue<'fd, S, C, M> {
    pub sqes: NonNull<S>,
    pub k_head: &'fd AtomicU32,
    pub k_tail: &'fd AtomicU32,
    pub mask: u32,
    pub size: u32,
    pub k_flags: &'fd AtomicU32,
    pub k_dropped: &'fd AtomicU32,

    _marker_: PhantomData<(C, M)>,
}

impl<S, C, M> SubmissionQueue<'_, S, C, M> {
    pub unsafe fn new(sq_mmap: &Mmap, sqes_mmap: &Mmap, params: &IoUringParams) -> Self {
        let IoUringParams { sq_off, .. } = params;

        unsafe {
            let sqes = sqes_mmap.ptr().cast();
            let k_head = sq_mmap.offset(sq_off.head).cast().as_ref();
            let k_tail = sq_mmap.offset(sq_off.tail).cast().as_ref();
            let mask = sq_mmap.offset(sq_off.ring_mask).cast().read();
            let size = sq_mmap.offset(sq_off.ring_entries).cast().read();
            let k_flags = sq_mmap.offset(sq_off.flags).cast().as_ref();
            let k_dropped = sq_mmap.offset(sq_off.dropped).cast().as_ref();
            SubmissionIndex::setup(sq_mmap, params);

            Self { sqes, k_head, k_tail, mask, size, k_flags, k_dropped, _marker_: PhantomData }
        }
    }

    pub fn flags(&self, order: Ordering) -> IoUringSqFlags {
        let bits = self.k_flags.load(order);
        IoUringSqFlags::from_bits_retain(bits)
    }

    pub fn dropped(&self) -> u32 {
        self.k_dropped.load(Ordering::Acquire)
    }

    pub fn need_wakeup(&self) -> bool {
        self.flags(Ordering::Relaxed).contains(IoUringSqFlags::NEED_WAKEUP)
    }

    pub fn cq_overflow(&self) -> bool {
        self.flags(Ordering::Relaxed).contains(IoUringSqFlags::CQ_OVERFLOW)
    }

    pub fn taskrun(&self) -> bool {
        self.flags(Ordering::Relaxed).contains(IoUringSqFlags::TASKRUN)
    }

    #[inline]
    pub const fn get_sqe(&self, idx: u32) -> NonNull<S> {
        // SAFETY: index masked
        unsafe { self.sqes.add((idx & self.mask) as usize) }
    }
}

impl<'fd, S, C, M> SubmissionQueue<'fd, S, C, M>
where
    M: Mode,
{
    #[inline]
    pub fn head(&self) -> u32 {
        M::get_sq_head(self)
    }

    #[inline]
    pub const fn tail(&self) -> u32 {
        // SAFETY: userspace set SubmissionQueue k_tail
        unsafe { *self.k_tail.as_ptr() }
    }

    #[inline]
    pub fn set_tail(&mut self, tail: u32) {
        M::set_sq_tail(self, tail);
    }

    pub fn submitter(&mut self) -> Submitter<'_, 'fd, S, C, M> {
        Submitter { head: self.head(), tail: self.tail(), queue: self }
    }
}

impl<S, C, M> Index<u32> for SubmissionQueue<'_, S, C, M> {
    type Output = S;

    #[inline]
    fn index(&self, index: u32) -> &Self::Output {
        // TODO: handle SQARRAY SubmissionIndex
        unsafe { self.get_sqe(index).as_ref() }
    }
}

impl<S, C, M> IndexMut<u32> for SubmissionQueue<'_, S, C, M> {
    #[inline]
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        unsafe { self.get_sqe(index).as_mut() }
    }
}
