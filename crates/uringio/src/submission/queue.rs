use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr::NonNull,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{
    arena::Arena,
    platform::iouring::{IoUringParams, IoUringSqFlags},
    submission::{index::SubmissionIndex, submitter::Submitter},
    uring::mode::Mode,
};

/// ## Submission Queue
#[derive(Debug)]
pub struct SubmissionQueue<'fd, A, M, S, C> {
    pub sqes: NonNull<S>,
    pub k_head: &'fd AtomicU32,
    pub k_tail: &'fd AtomicU32,
    pub mask: u32,
    pub size: u32,
    pub k_flags: &'fd AtomicU32,
    pub k_dropped: &'fd AtomicU32,

    _marker_: PhantomData<(A, M, C)>,
}

impl<A, M, S, C> SubmissionQueue<'_, A, M, S, C>
where
    A: Arena<M, S, C>,
{
    pub unsafe fn new(arena: &A, params: &IoUringParams) -> Self {
        let IoUringParams { sq_off, .. } = params;

        let sq = arena.sq();
        unsafe {
            let sqes = arena.sqes().cast();

            let k_head = sq.byte_add(sq_off.head as _).cast().as_ref();
            let k_tail = sq.byte_add(sq_off.tail as _).cast().as_ref();
            let mask = sq.byte_add(sq_off.ring_mask as _).cast().read();
            let size = sq.byte_add(sq_off.ring_entries as _).cast().read();
            let k_flags = sq.byte_add(sq_off.flags as _).cast().as_ref();
            let k_dropped = sq.byte_add(sq_off.dropped as _).cast().as_ref();
            SubmissionIndex::setup(sq, params);

            Self { sqes, k_head, k_tail, mask, size, k_flags, k_dropped, _marker_: PhantomData }
        }
    }
}

impl<A, M, S, C> SubmissionQueue<'_, A, M, S, C> {
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

impl<'fd, A, M, S, C> SubmissionQueue<'fd, A, M, S, C>
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

    pub fn submitter(&mut self) -> Submitter<'_, 'fd, A, M, S, C> {
        Submitter { head: self.head(), tail: self.tail(), queue: self }
    }
}

impl<A, M, S, C> Index<u32> for SubmissionQueue<'_, A, M, S, C> {
    type Output = S;

    #[inline]
    fn index(&self, index: u32) -> &Self::Output {
        // TODO: handle SQARRAY SubmissionIndex
        unsafe { self.get_sqe(index).as_ref() }
    }
}

impl<A, M, S, C> IndexMut<u32> for SubmissionQueue<'_, A, M, S, C> {
    #[inline]
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        unsafe { self.get_sqe(index).as_mut() }
    }
}
