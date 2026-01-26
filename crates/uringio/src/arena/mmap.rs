use std::{fmt::Debug, marker::PhantomData};

use crate::{
    arena::Arena,
    completion::entry::Cqe,
    platform::{
        iouring::{AsFd, IOURING_OFF_SQ_RING, IOURING_OFF_SQES, OwnedFd},
        mmap::{Mmap, Ptr},
    },
    shared::{error::Result, log::debug},
    submission::entry::Sqe,
    uring::args::UringArgs,
};

/// ## Mmap Arena
#[derive(Debug)]
pub struct MmapArena<'fd, M, S, C> {
    pub ring_mmap: Mmap,
    pub sqes_mmap: Mmap,

    _marker_: PhantomData<(&'fd OwnedFd, M, S, C)>,
}

impl<M, S, C> MmapArena<'_, M, S, C>
where
    S: Sqe,
    C: Cqe,
{
    pub fn new<Fd>(fd: &Fd, args: &UringArgs<Self, M, S, C>) -> Result<Self>
    where
        Fd: Debug + AsFd,
    {
        let ring_mem = args.ring_mem();
        let sqes_mem = args.sqes_mem();
        debug!("ring_mem: {ring_mem}, sqes_mem: {sqes_mem}");

        let ring_mmap = Mmap::new(fd, ring_mem, IOURING_OFF_SQ_RING)?;
        let sqes_mmap = Mmap::new(fd, sqes_mem, IOURING_OFF_SQES)?;

        Ok(Self { ring_mmap, sqes_mmap, _marker_: PhantomData })
    }
}

impl<M, S, C> Arena<M, S, C> for MmapArena<'_, M, S, C>
where
    S: Sqe,
    C: Cqe,
{
    fn sq(&self) -> Ptr {
        self.ring_mmap.ptr()
    }

    fn sqes(&self) -> Ptr {
        self.sqes_mmap.ptr()
    }

    fn cq(&self) -> Ptr {
        self.ring_mmap.ptr()
    }
}
