use std::sync::atomic::Ordering;

use crate::{
    completion::entry::Cqe,
    platform::iouring::{IoUringEnterFlags, IoUringSetupFlags},
    shared::constant::DEFAULT_SQ_POLL_IDLE,
    submission::{entry::Sqe, queue::SubmissionQueue},
    uring::args::SetupArgs,
};

#[derive(Debug)]
pub enum Ty {
    Iopoll,
    Sqpoll,
}

/// ## Mode
pub trait Mode: Sized {
    const TYPE: Ty;

    const SETUP_FLAG: IoUringSetupFlags = match Self::TYPE {
        Ty::Iopoll => IoUringSetupFlags::IOPOLL,
        Ty::Sqpoll => IoUringSetupFlags::SQPOLL,
    };

    const ENTER_FLAG: IoUringEnterFlags = match Self::TYPE {
        Ty::Iopoll => IoUringEnterFlags::GETEVENTS,
        Ty::Sqpoll => IoUringEnterFlags::empty(),
    };

    fn get_sq_head<S, C>(sq: &SubmissionQueue<'_, Self, S, C>) -> u32;

    fn set_sq_tail<S, C>(sq: &mut SubmissionQueue<'_, Self, S, C>, tail: u32);
}

/// ## Iopoll
#[derive(Debug)]
pub struct Iopoll;

impl Mode for Iopoll {
    const TYPE: Ty = Ty::Iopoll;

    #[inline]
    fn get_sq_head<S, C>(sq: &SubmissionQueue<'_, Self, S, C>) -> u32 {
        // SAFETY: userspace drive update in IOPOLL mode
        unsafe { *sq.k_head.as_ptr() }
    }

    #[inline]
    fn set_sq_tail<S, C>(sq: &mut SubmissionQueue<'_, Self, S, C>, tail: u32) {
        // SAFETY: userspace drive update in IOPOLL mode
        unsafe { *sq.k_tail.as_ptr() = tail }
    }
}

impl Iopoll {
    pub fn new<S, C>(entries: u32) -> SetupArgs<Self, S, C>
    where
        S: Sqe,
        C: Cqe,
    {
        SetupArgs::new(entries)
            .iopoll()
            .clamp()
            .submit_all()
            .coop_taskrun()
            .taskrun_flag()
            .single_issuer()
            .defer_taskrun()
            .no_sqarray()
            .hybrid_iopoll()
    }
}

/// ## Sqpoll
#[derive(Debug)]
pub struct Sqpoll;

impl Mode for Sqpoll {
    const TYPE: Ty = Ty::Sqpoll;

    #[inline]
    fn get_sq_head<S, C>(sq: &SubmissionQueue<'_, Self, S, C>) -> u32 {
        sq.k_head.load(Ordering::Acquire)
    }

    #[inline]
    fn set_sq_tail<S, C>(sq: &mut SubmissionQueue<'_, Self, S, C>, tail: u32) {
        sq.k_tail.store(tail, Ordering::Release);
    }
}

impl Sqpoll {
    pub fn new<S, C>(entries: u32) -> SetupArgs<Self, S, C>
    where
        S: Sqe,
        C: Cqe,
    {
        SetupArgs::new(entries)
            .sqpoll(DEFAULT_SQ_POLL_IDLE)
            .clamp()
            .submit_all()
            .single_issuer()
            .no_sqarray()
    }
}
