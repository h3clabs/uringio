pub mod args;
pub mod enter;
pub mod feat;
pub mod mode;

use crate::{
    arena::Arena,
    completion::{
        collector::Collector,
        entry::{Cqe, Cqe16, Cqe32, CqeMix},
        queue::CompletionQueue,
    },
    platform::iouring::OwnedFd,
    shared::error::Result,
    submission::{
        entry::{Sqe, Sqe64, Sqe128, SqeMix},
        queue::SubmissionQueue,
        submitter::Submitter,
    },
    uring::{args::UringArgs, enter::UringEnter, mode::Mode},
};

/// ## Uring
#[derive(Debug)]
pub struct Uring<'fd, A, M, S, C> {
    pub enter: UringEnter<'fd, A, M, S, C>,
    pub sq: SubmissionQueue<'fd, A, M, S, C>,
    pub cq: CompletionQueue<'fd, A, M, S, C>,
    arena: A,
}

impl<'fd, A, M, S, C> Uring<'fd, A, M, S, C>
where
    A: Arena<M, S, C>,
    M: Mode,
    S: Sqe,
    C: Cqe,
{
    pub fn new(
        fd: &'fd OwnedFd,
        args: &UringArgs<A, M, S, C>,
        arena: A,
    ) -> Result<Uring<'fd, A, M, S, C>> {
        unsafe {
            let enter = UringEnter::new(fd, args);
            let sq = SubmissionQueue::new(&arena, args);
            let cq = CompletionQueue::new(&arena, args);
            Ok(Uring { enter, sq, cq, arena })
        }
    }
}

impl<'fd, A, M, S, C> Uring<'fd, A, M, S, C>
where
    M: Mode,
    S: Sqe,
    C: Cqe,
{
    pub fn register(mut self) -> Result<Self> {
        self.enter.register_ring_fd()?;
        Ok(self)
    }

    pub fn arena(&self) -> &A {
        &self.arena
    }

    pub fn submitter(&mut self) -> Submitter<'_, 'fd, A, M, S, C> {
        self.sq.submitter()
    }

    pub fn collector(&mut self) -> Collector<'_, 'fd, A, M, S, C> {
        self.cq.collector()
    }

    pub fn borrow(
        &mut self,
    ) -> (
        &mut UringEnter<'fd, A, M, S, C>,
        Submitter<'_, 'fd, A, M, S, C>,
        Collector<'_, 'fd, A, M, S, C>,
    ) {
        (&mut self.enter, self.sq.submitter(), self.cq.collector())
    }
}

pub type UringIo<'fd, A, M> = Uring<'fd, A, M, Sqe64, Cqe16>;

pub type Uring128<'fd, A, M> = Uring<'fd, A, M, Sqe128, Cqe32>;

pub type UringMix<'fd, A, M> = Uring<'fd, A, M, SqeMix, CqeMix>;

// TODO: UringIo self referential lifetime
// #[derive(Debug)]
// pub struct UringIo<S, C> {
//     pub fd: UringFd<S, C>,
//     pub uring: Uring<'fd, S, C>,
// }
