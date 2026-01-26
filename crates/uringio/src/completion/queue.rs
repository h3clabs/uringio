use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr::NonNull,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{
    arena::Arena,
    completion::collector::Collector,
    platform::iouring::{IoUringCqFlags, IoUringParams, IoUringSqFlags},
    uring::mode::Mode,
};

/// ## Completion Queue
#[derive(Debug)]
pub struct CompletionQueue<'fd, A, M, S, C> {
    pub cqes: NonNull<C>,
    pub k_head: &'fd AtomicU32,
    pub k_tail: &'fd AtomicU32,
    pub mask: u32,
    pub size: u32,
    pub k_flags: &'fd AtomicU32,
    pub k_overflow: &'fd AtomicU32,

    pub k_sq_flags: &'fd AtomicU32,

    _marker_: PhantomData<(A, M, S)>,
}

impl<A, M, S, C> CompletionQueue<'_, A, M, S, C>
where
    A: Arena<M, S, C>,
{
    pub unsafe fn new(arena: &A, params: &IoUringParams) -> Self {
        let IoUringParams { sq_off, cq_off, .. } = params;

        let cq = arena.cq();
        unsafe {
            let cqes = cq.byte_add(cq_off.cqes as _).cast::<C>();
            let k_head = cq.byte_add(cq_off.head as _).cast().as_ref();
            let k_tail = cq.byte_add(cq_off.tail as _).cast().as_ref();
            let mask = cq.byte_add(cq_off.ring_mask as _).cast().read();
            let size = cq.byte_add(cq_off.ring_entries as _).cast().read();
            let k_flags = cq.byte_add(cq_off.flags as _).cast().as_ref();
            let k_overflow = cq.byte_add(cq_off.overflow as _).cast().as_ref();
            let k_sq_flags = cq.byte_add(sq_off.flags as _).cast().as_ref();

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
}

impl<A, M, S, C> CompletionQueue<'_, A, M, S, C> {
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

impl<'fd, A, M, S, C> CompletionQueue<'fd, A, M, S, C>
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

    pub fn collector(&mut self) -> Collector<'_, 'fd, A, M, S, C> {
        Collector { head: self.head(), tail: self.tail(), queue: self }
    }
}

impl<A, M, S, C> Index<u32> for CompletionQueue<'_, A, M, S, C> {
    type Output = C;

    #[inline]
    fn index(&self, index: u32) -> &Self::Output {
        // TODO: handle SQARRAY SubmissionIndex
        unsafe { self.get_cqe(index).as_ref() }
    }
}

impl<A, M, S, C> IndexMut<u32> for CompletionQueue<'_, A, M, S, C> {
    #[inline]
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        unsafe { self.get_cqe(index).as_mut() }
    }
}
