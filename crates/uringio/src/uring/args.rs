use std::{marker::PhantomData, ops::Deref};

use crate::{
    arena::{huge::HugeArena, mmap::MmapArena},
    completion::entry::Cqe,
    platform::iouring::{
        AsRawFd, IOURING_IO_RINGS_SIZE, IoUringParams, IoUringSetupFlags, OwnedFd, io_uring_setup,
    },
    shared::{error::Result, log::debug},
    submission::entry::Sqe,
    uring::mode::Mode,
};

#[derive(Debug)]
#[repr(transparent)]
pub struct UringArgs<A, M, S, C> {
    pub(crate) params: IoUringParams,

    _marker_: PhantomData<(A, M, S, C)>,
}

impl<A, M, S, C> UringArgs<A, M, S, C>
where
    M: Mode,
    S: Sqe,
    C: Cqe,
{
    pub fn new(entries: u32) -> Self {
        let mut params = IoUringParams::default();
        params.sq_entries = entries;
        params.flags = S::SETUP_FLAG | C::SETUP_FLAG | M::SETUP_FLAG;

        UringArgs { params, _marker_: PhantomData }
    }

    pub fn sqsize(mut self, entries: u32) -> Self {
        self.params.sq_entries = entries;
        self
    }

    pub fn iopoll(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::IOPOLL;
        self
    }

    pub fn sqpoll(mut self, idle: u32) -> Self {
        self.params.flags |= IoUringSetupFlags::SQPOLL;
        self.params.sq_thread_idle = idle;
        self
    }

    pub fn sqpoll_cpu(mut self, cpu: u32) -> Self {
        self.params.flags |= IoUringSetupFlags::SQ_AFF;
        self.params.sq_thread_cpu = cpu;
        self
    }

    pub fn cqsize(mut self, entries: u32) -> Self {
        self.params.flags |= IoUringSetupFlags::CQSIZE;
        self.params.cq_entries = entries;
        self
    }

    pub fn clamp(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::CLAMP;
        self
    }

    pub fn attach_wq<Fd>(mut self, fd: &Fd) -> Self
    where
        Fd: AsRawFd,
    {
        self.params.flags |= IoUringSetupFlags::ATTACH_WQ;
        self.params.wq_fd = fd.as_raw_fd();
        self
    }

    pub fn r_disabled(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::R_DISABLED;
        self
    }

    pub fn submit_all(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::SUBMIT_ALL;
        self
    }

    // Must use with IOPOLL
    pub fn coop_taskrun(mut self) -> Self {
        debug_assert!(self.flags.contains(IoUringSetupFlags::IOPOLL));

        self.params.flags |= IoUringSetupFlags::COOP_TASKRUN;
        self
    }

    // Must use with IOPOLL
    pub fn taskrun_flag(mut self) -> Self {
        debug_assert!(self.flags.contains(IoUringSetupFlags::IOPOLL));

        self.params.flags |= IoUringSetupFlags::TASKRUN_FLAG;
        self
    }

    pub fn single_issuer(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::SINGLE_ISSUER;
        self
    }

    // Must use with IOPOLL | SINGLE_ISSUER
    pub fn defer_taskrun(mut self) -> Self {
        debug_assert!(self.flags.contains(IoUringSetupFlags::IOPOLL));
        debug_assert!(self.flags.contains(IoUringSetupFlags::SINGLE_ISSUER));

        self.params.flags |= IoUringSetupFlags::DEFER_TASKRUN;
        self
    }

    // Must use with NO_MMAP
    pub fn registered_fd_only(mut self) -> Self {
        debug_assert!(self.flags.contains(IoUringSetupFlags::NO_MMAP));

        self.params.flags |= IoUringSetupFlags::REGISTERED_FD_ONLY;
        self
    }

    pub fn no_sqarray(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::NO_SQARRAY;
        self
    }

    // Must use with IOPOLL
    pub fn hybrid_iopoll(mut self) -> Self {
        debug_assert!(self.flags.contains(IoUringSetupFlags::IOPOLL));

        self.params.flags |= IoUringSetupFlags::HYBRID_IOPOLL;
        self
    }
}

impl<'fd, M, S, C> UringArgs<MmapArena<'fd, M, S, C>, M, S, C>
where
    M: Mode,
    S: Sqe,
    C: Cqe,
{
    pub fn setup(mut self) -> Result<(OwnedFd, Self, MmapArena<'fd, M, S, C>)> {
        debug!("setup args: {:?}", self.params);

        let fd = unsafe { io_uring_setup(self.sq_entries, &mut self.params)? };
        debug!("uring fd: {fd:?}, params: {:?}", self.params);

        #[cfg(feature = "features-checker")]
        {
            use crate::uring::feat::check_setup_features;
            check_setup_features(self.features)?;
        }

        let arena = MmapArena::new(&fd, &self)?;

        Ok((fd, self, arena))
    }

    pub fn no_mmap(mut self) -> UringArgs<HugeArena<M, S, C>, M, S, C> {
        self.params.flags |= IoUringSetupFlags::NO_MMAP;
        UringArgs { params: self.params, _marker_: PhantomData }
    }
}

impl<M, S, C> UringArgs<HugeArena<M, S, C>, M, S, C>
where
    M: Mode,
    S: Sqe,
    C: Cqe,
{
    pub fn setup(mut self) -> Result<(OwnedFd, Self, HugeArena<M, S, C>)> {
        debug!("setup args: {:?}", self.params);

        let arena = HugeArena::setup(&mut self)?;

        let fd = unsafe { io_uring_setup(self.sq_entries, &mut self.params)? };
        debug!("uring fd: {fd:?}, params: {:?}", self.params);

        #[cfg(feature = "features-checker")]
        {
            use crate::uring::feat::check_setup_features;
            check_setup_features(self.features)?;
        }
        debug_assert!(self.cq_off.cqes as usize == IOURING_IO_RINGS_SIZE);

        Ok((fd, self, arena))
    }
}

impl<A, M, S, C> UringArgs<A, M, S, C>
where
    S: Sqe,
    C: Cqe,
{
    pub fn ring_mem(&self) -> usize {
        self.cq_off.cqes as usize + self.cqes_mem() + self.sq_indices_mem()
    }

    // Unsafe: return 0 when NO_SQARRAY
    pub unsafe fn sq_mem(&self) -> usize {
        self.sq_off.array as usize + self.sq_indices_mem()
    }

    pub fn sq_indices_mem(&self) -> usize {
        if self.flags.contains(IoUringSetupFlags::NO_SQARRAY) {
            0
        } else {
            self.sq_entries as usize * size_of::<u32>()
        }
    }

    pub fn sqes_mem(&self) -> usize {
        self.sq_entries as usize * S::SETUP_SQE_SIZE
    }

    pub fn cq_mem(&self) -> usize {
        self.cq_off.cqes as usize + self.cqes_mem()
    }

    pub fn cqes_mem(&self) -> usize {
        self.cq_entries as usize * C::SETUP_CQE_SIZE
    }
}

impl<A, M, S, C> Deref for UringArgs<A, M, S, C> {
    type Target = IoUringParams;

    fn deref(&self) -> &Self::Target {
        &self.params
    }
}
