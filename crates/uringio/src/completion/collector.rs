use std::sync::{atomic, atomic::Ordering};

use crate::{
    completion::queue::CompletionQueue,
    platform::iouring::{IoUringEnterFlags, IoUringSqFlags},
    shared::error::Result,
    uring::{
        enter::UringEnter,
        mode::{Mode, Sqpoll},
    },
};

#[derive(Debug)]
pub struct Collector<'c, 'fd, M, S, C>
where
    M: Mode,
{
    pub(crate) head: u32,
    pub(crate) tail: u32,
    pub queue: &'c mut CompletionQueue<'fd, M, S, C>,
}

impl<M, S, C> Collector<'_, '_, M, S, C>
where
    M: Mode,
{
    #[inline]
    pub fn update_head(&mut self) {
        self.queue.set_head(self.head);
    }

    #[inline]
    pub fn update_tail(&mut self) {
        self.tail = self.queue.tail();
    }

    pub fn update(&mut self) {
        self.update_head();
        self.update_tail();
    }

    #[inline]
    pub const fn is_full(&self) -> bool {
        self.size() == self.queue.size
    }

    #[inline]
    pub const fn size(&self) -> u32 {
        self.tail.wrapping_sub(self.head)
    }
}

impl<M, S, C> Drop for Collector<'_, '_, M, S, C>
where
    M: Mode,
{
    fn drop(&mut self) {
        self.update_head();
    }
}

impl<'c, M, S, C> Iterator for Collector<'c, '_, M, S, C>
where
    M: Mode,
{
    type Item = &'c C;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.head == self.tail {
            return None;
        }

        let cqe = self.queue.get_cqe(self.head);
        self.head = self.head.wrapping_add(1);
        Some(unsafe { cqe.as_ref() })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<M, S, C> ExactSizeIterator for Collector<'_, '_, M, S, C>
where
    M: Mode,
{
    #[inline]
    fn len(&self) -> usize {
        #![allow(clippy::as_conversions)]
        self.size() as usize
    }
}

impl<'fd, S, C> Collector<'_, 'fd, Sqpoll, S, C> {
    pub fn flush(
        &mut self,
        enter: &mut UringEnter<'fd, Sqpoll, S, C>,
        min_complete: u32,
    ) -> Result<u32> {
        // TODO: void fence(SeqCst): https://github.com/axboe/liburing/issues/541
        atomic::fence(Ordering::SeqCst);
        let sq_flags = self.queue.sq_flags(Ordering::Relaxed);
        let sq_wakeup = sq_flags.contains(IoUringSqFlags::NEED_WAKEUP);
        let cq_overflow = sq_flags.contains(IoUringSqFlags::CQ_OVERFLOW);
        let enter_getevents = min_complete > 0 || cq_overflow;

        let mut flags = IoUringEnterFlags::default();

        if sq_wakeup {
            flags.insert(IoUringEnterFlags::SQ_WAKEUP);
        } else if !enter_getevents {
            // IORING_FEAT_NODROP enabled since kernel 5.5
            // https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/commit/?id=1d7bb1d50fb4dc141c7431cc21fdd24ffcc83c76
            return Ok(0);
        }

        if enter_getevents {
            // IORING_ENTER_GETEVENTS call io_cqring_do_overflow_flush()
            flags.insert(IoUringEnterFlags::GETEVENTS);
        }

        enter.enter(0, min_complete, flags)
    }
}
