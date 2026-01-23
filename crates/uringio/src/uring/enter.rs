use std::{io::Result, marker::PhantomData};

use crate::{
    platform::iouring::{
        AsFd, BorrowedFd, IoUringEnterFlags, IoUringFeatureFlags, OwnedFd, io_uring_enter,
    },
    uring::{args::UringArgs, mode::Mode},
};

#[derive(Debug)]
pub struct UringEnter<'fd, S, C, M> {
    pub(crate) enter_fd: BorrowedFd<'fd>,
    // TODO: init flags
    pub(crate) enter_flags: IoUringEnterFlags,
    pub(crate) features: IoUringFeatureFlags,

    _marker_: PhantomData<(S, C, M)>,
}

impl<'fd, S, C, M> UringEnter<'fd, S, C, M>
where
    M: Mode,
{
    pub fn new(fd: &'fd OwnedFd, args: &UringArgs<S, C, M>) -> Self {
        Self {
            enter_fd: fd.as_fd(),
            enter_flags: M::ENTER_FLAG,
            features: args.features,
            _marker_: PhantomData,
        }
    }
}

impl<S, C, M> UringEnter<'_, S, C, M> {
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

impl<S, C, M> Drop for UringEnter<'_, S, C, M> {
    fn drop(&mut self) {
        if self.is_ring_registered() {
            unsafe {
                let _ = self.unregister_ring_fd();
            }
        }
    }
}
