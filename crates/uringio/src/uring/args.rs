use std::{marker::PhantomData, ops::Deref};

use crate::{
    completion::entry::Cqe,
    platform::iouring::{AsRawFd, IoUringParams, IoUringSetupFlags, OwnedFd, io_uring_setup},
    shared::error::Result,
    submission::entry::Sqe,
    uring::mode::Mode,
};

#[derive(Debug)]
#[repr(transparent)]
pub struct SetupArgs<M, S, C> {
    pub params: IoUringParams,

    _marker_: PhantomData<(M, S, C)>,
}

impl<M, S, C> SetupArgs<M, S, C>
where
    M: Mode,
    S: Sqe,
    C: Cqe,
{
    pub fn new(entries: u32) -> Self {
        let mut params = IoUringParams::default();
        params.flags |= S::SETUP_FLAG;
        params.flags |= C::SETUP_FLAG;
        params.flags |= M::SETUP_FLAG;
        params.sq_entries = entries;
        SetupArgs { params, _marker_: PhantomData }
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
        self.params.flags |= IoUringSetupFlags::COOP_TASKRUN;
        self
    }

    // Must use with IOPOLL
    pub fn taskrun_flag(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::TASKRUN_FLAG;
        self
    }

    pub fn single_issuer(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::SINGLE_ISSUER;
        self
    }

    // Must use with IOPOLL | SINGLE_ISSUER
    pub fn defer_taskrun(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::DEFER_TASKRUN;
        self
    }

    pub fn no_mmap(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::NO_MMAP;
        // TODO: setup hugepage mmap
        self
    }

    // Must use with NO_MMAP
    pub fn registered_fd_only(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::REGISTERED_FD_ONLY;
        self
    }

    pub fn no_sqarray(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::NO_SQARRAY;
        self
    }

    // Must use with IOPOLL
    pub fn hybrid_iopoll(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::HYBRID_IOPOLL;
        self
    }

    pub fn setup(self) -> Result<(OwnedFd, UringArgs<M, S, C>)> {
        let Self { mut params, .. } = self;
        let fd = unsafe { io_uring_setup(params.sq_entries, &mut params)? };

        #[cfg(feature = "features-checker")]
        {
            use crate::uring::feat::check_setup_features;
            check_setup_features(params.features)?;
        }

        let args = UringArgs { params, _marker_: PhantomData };
        Ok((fd, args))
    }
}

#[derive(Debug)]
pub struct UringArgs<M, S, C> {
    params: IoUringParams,
    _marker_: PhantomData<(M, S, C)>,
}

impl<M, S, C> UringArgs<M, S, C>
where
    S: Sqe,
    C: Cqe,
{
    pub fn sq_size(&self) -> usize {
        self.sq_off.array as usize + self.sq_indices_size()
    }

    pub fn sq_indices_size(&self) -> usize {
        if self.flags.contains(IoUringSetupFlags::NO_SQARRAY) {
            0
        } else {
            self.sq_entries as usize * size_of::<u32>()
        }
    }

    pub fn sqes_size(&self) -> usize {
        self.sq_entries as usize * S::SETUP_SQE_SIZE
    }

    pub fn cq_size(&self) -> usize {
        self.cq_off.cqes as usize + self.cqes_size()
    }

    pub fn cqes_size(&self) -> usize {
        self.cq_entries as usize * C::SETUP_CQE_SIZE
    }
}

impl<M, S, C> Deref for UringArgs<M, S, C> {
    type Target = IoUringParams;

    fn deref(&self) -> &Self::Target {
        &self.params
    }
}
