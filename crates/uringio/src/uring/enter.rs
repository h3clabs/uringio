use std::{io::Result, marker::PhantomData};

use crate::{
    platform::iouring::{
        AsFd, BorrowedFd, IoUringEnterFlags, IoUringFeatureFlags, OwnedFd, io_uring_enter,
    },
    uring::{args::UringArgs, mode::Mode},
};

#[derive(Debug)]
pub struct UringEnter<'fd, A, M, S, C> {
    pub(crate) enter_fd: BorrowedFd<'fd>,
    // TODO: init flags
    pub(crate) enter_flags: IoUringEnterFlags,
    pub(crate) features: IoUringFeatureFlags,

    _marker_: PhantomData<(A, M, S, C)>,
}

impl<'fd, A, M, S, C> UringEnter<'fd, A, M, S, C>
where
    M: Mode,
{
    pub fn new(fd: &'fd OwnedFd, args: &UringArgs<A, M, S, C>) -> Self {
        Self {
            enter_fd: fd.as_fd(),
            enter_flags: M::ENTER_FLAG,
            features: args.features,
            _marker_: PhantomData,
        }
    }
}

impl<A, M, S, C> UringEnter<'_, A, M, S, C> {
    #[inline]
    pub fn features(&self) -> &IoUringFeatureFlags {
        &self.features
    }

    pub fn set_iowait(&mut self, enable: bool) {
        #[cfg(feature = "features-checker")]
        {
            if !self.features.contains(IoUringFeatureFlags::NO_IOWAIT) {
                return;
            }
        }

        if enable {
            self.enter_flags.insert(IoUringEnterFlags::NO_IOWAIT);
        } else {
            self.enter_flags.remove(IoUringEnterFlags::NO_IOWAIT);
        }
    }

    #[inline]
    pub fn enter(
        &self,
        to_submit: u32,
        min_complete: u32,
        flags: IoUringEnterFlags,
    ) -> Result<u32> {
        Ok(unsafe {
            io_uring_enter(self.enter_fd, to_submit, min_complete, self.enter_flags | flags)?
        })
    }
}

impl<A, M, S, C> Drop for UringEnter<'_, A, M, S, C> {
    fn drop(&mut self) {
        if self.is_ring_registered() {
            unsafe {
                let _ = self.unregister_ring_fd();
            }
        }
    }
}
