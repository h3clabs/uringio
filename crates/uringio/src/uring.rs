pub mod args;
pub mod enter;
pub mod feat;
pub mod mode;

use crate::{
    completion::{
        collector::Collector,
        entry::{Cqe, Cqe16, Cqe32, CqeMix},
        queue::CompletionQueue,
    },
    mmap_arena::MmapArena,
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
pub struct Uring<'fd, M, S, C> {
    pub enter: UringEnter<'fd, M, S, C>,
    pub sq: SubmissionQueue<'fd, M, S, C>,
    pub cq: CompletionQueue<'fd, M, S, C>,
    arena: MmapArena<'fd, M, S, C>,
}

impl<'fd, M, S, C> Uring<'fd, M, S, C>
where
    M: Mode,
    S: Sqe,
    C: Cqe,
{
    pub fn new(fd: &'fd OwnedFd, args: &UringArgs<M, S, C>) -> Result<Self> {
        unsafe {
            let arena = MmapArena::new(fd, args)?;

            let enter = UringEnter::new(fd, args);
            let sq = SubmissionQueue::new(&arena.sq_mmap, &arena.sqes_mmap, args);
            let cq = CompletionQueue::new(&arena.sq_mmap, arena.cq_mmap(), args);
            Ok(Uring { enter, sq, cq, arena })
        }
    }

    pub fn register(mut self) -> Result<Self> {
        self.enter.register_ring_fd()?;
        Ok(self)
    }

    pub fn arena(&self) -> &MmapArena<'fd, M, S, C> {
        &self.arena
    }

    pub fn submitter(&mut self) -> Submitter<'_, 'fd, M, S, C> {
        self.sq.submitter()
    }

    pub fn collector(&mut self) -> Collector<'_, 'fd, M, S, C> {
        self.cq.collector()
    }

    pub fn borrow(
        &mut self,
    ) -> (&mut UringEnter<'fd, M, S, C>, Submitter<'_, 'fd, M, S, C>, Collector<'_, 'fd, M, S, C>)
    {
        (&mut self.enter, self.sq.submitter(), self.cq.collector())
    }
}

pub type UringIo<'fd, M> = Uring<'fd, M, Sqe64, Cqe16>;

pub type Uring128<'fd, M> = Uring<'fd, M, Sqe128, Cqe32>;

pub type UringMix<'fd, M> = Uring<'fd, M, SqeMix, CqeMix>;

// TODO: UringIo self referential lifetime
// #[derive(Debug)]
// pub struct UringIo<S, C> {
//     pub fd: UringFd<S, C>,
//     pub uring: Uring<'fd, S, C>,
// }
