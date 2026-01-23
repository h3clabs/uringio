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
pub struct Uring<'fd, S, C, M> {
    pub enter: UringEnter<'fd, S, C, M>,
    pub sq: SubmissionQueue<'fd, S, C, M>,
    pub cq: CompletionQueue<'fd, S, C, M>,
    arena: MmapArena<'fd, S, C, M>,
}

impl<'fd, S, C, M> Uring<'fd, S, C, M>
where
    S: Sqe,
    C: Cqe,
    M: Mode,
{
    pub fn new(fd: &'fd OwnedFd, args: &UringArgs<S, C, M>) -> Result<Self> {
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

    pub fn arena(&self) -> &MmapArena<'fd, S, C, M> {
        &self.arena
    }

    pub fn submitter(&mut self) -> Submitter<'_, 'fd, S, C, M> {
        self.sq.submitter()
    }

    pub fn collector(&mut self) -> Collector<'_, 'fd, S, C, M> {
        self.cq.collector()
    }

    pub fn borrow(
        &mut self,
    ) -> (&mut UringEnter<'fd, S, C, M>, Submitter<'_, 'fd, S, C, M>, Collector<'_, 'fd, S, C, M>)
    {
        (&mut self.enter, self.sq.submitter(), self.cq.collector())
    }
}

pub type UringIo<'fd, M> = Uring<'fd, Sqe64, Cqe16, M>;

pub type Uring128<'fd, M> = Uring<'fd, Sqe128, Cqe32, M>;

pub type UringMix<'fd, M> = Uring<'fd, SqeMix, CqeMix, M>;

// TODO: UringIo self referential lifetime
// #[derive(Debug)]
// pub struct UringIo<S, C> {
//     pub fd: UringFd<S, C>,
//     pub uring: Uring<'fd, S, C>,
// }
