use crate::{
    completion::entry::{Cqe16, Cqe32, CqeMix},
    operator::{Op, nop::Nop128},
    platform::iouring::IoUringEnterFlags,
    shared::{
        error::Result,
        null::{NULL, Null},
    },
    submission::{
        entry::{FixSqe, Sqe64, Sqe128, SqeMix},
        queue::SubmissionQueue,
    },
    uring::{
        enter::UringEnter,
        mode::{Iopoll, Mode, Sqpoll},
    },
};

/// ## Submitter
#[derive(Debug)]
pub struct Submitter<'s, 'fd, S, C, M>
where
    M: Mode,
{
    pub(crate) head: u32,
    pub(crate) tail: u32,
    pub queue: &'s mut SubmissionQueue<'fd, S, C, M>,
}

impl<S, C, M> Submitter<'_, '_, S, C, M>
where
    M: Mode,
{
    fn push_impl<T>(&mut self, sqe: T) -> Result<Null, T>
    where
        T: Into<S> + FixSqe,
    {
        if self.is_full() {
            return Err(sqe)
        }

        self.queue[self.tail] = sqe.into();
        self.tail = self.tail.wrapping_add(1);

        Ok(NULL)
    }

    #[inline]
    pub fn update_head(&mut self) {
        self.head = self.queue.head();
    }

    #[inline]
    pub fn update_tail(&mut self) {
        self.queue.set_tail(self.tail);
    }

    pub fn update(&mut self) {
        self.update_head();
        self.update_tail();
    }

    #[inline]
    pub const fn size(&self) -> u32 {
        self.tail.wrapping_sub(self.head)
    }

    #[inline]
    pub const fn is_full(&self) -> bool {
        self.size() == self.queue.size
    }
}

impl<S, C, M> Drop for Submitter<'_, '_, S, C, M>
where
    M: Mode,
{
    fn drop(&mut self) {
        self.update_tail();
    }
}

pub trait Submit<T> {
    fn push(&mut self, item: T) -> Result<Null, T>;
}

// Submit to Sqe64 Queue
impl<M> Submit<Sqe64> for Submitter<'_, '_, Sqe64, Cqe16, M>
where
    M: Mode,
{
    fn push(&mut self, sqe: Sqe64) -> Result<Null, Sqe64> {
        self.push_impl(sqe)
    }
}

// Submit to Sqe128 Queue
impl<M> Submit<Sqe128> for Submitter<'_, '_, Sqe128, Cqe32, M>
where
    M: Mode,
{
    fn push(&mut self, sqe: Sqe128) -> Result<Null, Sqe128> {
        self.push_impl(sqe)
    }
}

// Submit Sqe64 to SqeMix Queue
impl<M> Submit<Sqe64> for Submitter<'_, '_, SqeMix, CqeMix, M>
where
    M: Mode,
{
    fn push(&mut self, sqe: Sqe64) -> Result<Null, Sqe64> {
        self.push_impl(sqe)
    }
}

// Submit Sqe128 to SqeMix Queue
impl<M> Submit<Sqe128> for Submitter<'_, '_, SqeMix, CqeMix, M>
where
    M: Mode,
{
    fn push(&mut self, sqe: Sqe128) -> Result<Null, Sqe128> {
        // Sqe128 take 2 slots
        if self.size() + 2 > self.queue.size {
            return Err(sqe)
        }

        // Padding with IORING_OP_NOP128
        if self.tail.wrapping_add(1) & self.queue.mask == 0 {
            if self.size() + 3 > self.queue.size {
                return Err(sqe)
            }
            // Nop128 slot checked
            let _ = self.push(Nop128::new().skip_cqe());
        }

        unsafe { self.queue.get_sqe(self.tail).cast::<Sqe128>().write(sqe) };
        self.tail = self.tail.wrapping_add(2);

        Ok(NULL)
    }
}

// Submit Op
impl<T, S, C, M> Submit<T> for Submitter<'_, '_, S, C, M>
where
    M: Mode,
    T: Op + Into<S>,
{
    fn push(&mut self, op: T) -> Result<Null, T> {
        self.push_impl(op)
    }
}

impl<'fd, S, C> Submitter<'_, 'fd, S, C, Iopoll> {
    pub fn submit(
        &mut self,
        enter: &mut UringEnter<'fd, S, C, Iopoll>,
        min_complete: u32,
    ) -> Result<u32> {
        self.update();

        enter.enter(self.size(), min_complete, IoUringEnterFlags::GETEVENTS)
    }
}

impl<S, C> Submitter<'_, '_, S, C, Sqpoll> {
    pub fn submit(&mut self) {
        self.update();
    }
}
