use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr::NonNull,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{
    completion::collector::Collector,
    platform::{
        iouring::{IoUringCqFlags, IoUringParams, IoUringSqFlags},
        mmap::Mmap,
    },
    uring::mode::Mode,
};

/// ## Completion Queue
#[derive(Debug)]
pub struct CompletionQueue<'fd, M, S, C> {
    pub cqes: NonNull<C>,
    pub k_head: &'fd AtomicU32,
    pub k_tail: &'fd AtomicU32,
    pub mask: u32,
    pub size: u32,
    pub k_flags: &'fd AtomicU32,
    pub k_overflow: &'fd AtomicU32,

    pub k_sq_flags: &'fd AtomicU32,

    _marker_: PhantomData<(M, S)>,
}

impl<M, S, C> CompletionQueue<'_, M, S, C> {
    pub const unsafe fn new(sq_mmap: &Mmap, cq_mmap: &Mmap, params: &IoUringParams) -> Self {
        let IoUringParams { sq_off, cq_off, .. } = params;

        unsafe {
            let cqes = cq_mmap.offset(cq_off.cqes).cast();
            let k_head = cq_mmap.offset(cq_off.head).cast().as_ref();
            let k_tail = cq_mmap.offset(cq_off.tail).cast().as_ref();
            let mask = cq_mmap.offset(cq_off.ring_mask).cast().read();
            let size = cq_mmap.offset(cq_off.ring_entries).cast().read();
            let k_flags = cq_mmap.offset(cq_off.flags).cast().as_ref();
            let k_overflow = cq_mmap.offset(cq_off.overflow).cast().as_ref();

            let k_sq_flags = sq_mmap.offset(sq_off.flags).cast().as_ref();

            Self {
                cqes,
                k_head,
                k_tail,
                mask,
                size,
                k_flags,
                k_overflow,
                k_sq_flags,
                _marker_: PhantomData,
            }
        }
    }

    pub fn flags(&self, order: Ordering) -> IoUringCqFlags {
        let bits = self.k_flags.load(order);
        IoUringCqFlags::from_bits_retain(bits)
    }

    pub fn sq_flags(&self, order: Ordering) -> IoUringSqFlags {
        let bits = self.k_sq_flags.load(order);
        IoUringSqFlags::from_bits_retain(bits)
    }

    pub fn overflow(&self) -> u32 {
        self.k_overflow.load(Ordering::Acquire)
    }

    #[inline]
    pub const fn get_cqe(&self, idx: u32) -> NonNull<C> {
        // SAFETY: index masked
        #![allow(clippy::as_conversions)]
        unsafe { self.cqes.add((idx & self.mask) as usize) }
    }
}

impl<'fd, M, S, C> CompletionQueue<'fd, M, S, C>
where
    M: Mode,
{
    #[inline]
    pub const fn head(&self) -> u32 {
        // SAFETY: userspace set CompletionQueue k_head
        unsafe { *self.k_head.as_ptr() }
    }

    #[inline]
    pub fn tail(&self) -> u32 {
        self.k_tail.load(Ordering::Acquire)
    }

    #[inline]
    pub fn set_head(&mut self, head: u32) {
        self.k_head.store(head, Ordering::Release);
    }

    pub fn collector(&mut self) -> Collector<'_, 'fd, M, S, C> {
        Collector { head: self.head(), tail: self.tail(), queue: self }
    }
}

impl<M, S, C> Index<u32> for CompletionQueue<'_, M, S, C> {
    type Output = C;

    #[inline]
    fn index(&self, index: u32) -> &Self::Output {
        // TODO: handle SQARRAY SubmissionIndex
        unsafe { self.get_cqe(index).as_ref() }
    }
}

impl<M, S, C> IndexMut<u32> for CompletionQueue<'_, M, S, C> {
    #[inline]
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        unsafe { self.get_cqe(index).as_mut() }
    }
}
