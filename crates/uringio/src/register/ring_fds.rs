use crate::{
    platform::iouring::{
        AsRawFd, BorrowedFd, IoUringEnterFlags, IoUringFeatureFlags,
        IoUringRegisterOp::{RegisterRingFds, UnregisterRingFds},
        IoUringRsrcUpdate, io_uring_register,
    },
    register::args::{RegisterArgs, RegisterRingFd},
    shared::{
        error::{Result, err},
        null::{NULL, Null},
    },
    uring::enter::UringEnter,
};

impl<M, S, C> UringEnter<'_, M, S, C> {
    #[inline]
    pub fn is_ring_registered(&self) -> bool {
        self.enter_flags.contains(IoUringEnterFlags::REGISTERED_RING)
    }

    pub fn register_ring_fd(&mut self) -> Result<Null> {
        #[cfg(feature = "features-checker")]
        {
            if !self.features.contains(IoUringFeatureFlags::REG_REG_RING) {
                return err!("Feature REG_REG_RING Invalid");
            }
        }

        if self.is_ring_registered() {
            return err!("Ring fd registered");
        }

        #[allow(unused_mut)]
        let mut args = IoUringRsrcUpdate::new(self.enter_fd.as_raw_fd());
        // SAFETY: asm options !readonly
        let num = unsafe { io_uring_register(self.enter_fd, RegisterRingFds, args.as_ptr(), 1)? };

        if num != 1 {
            return err!("Failed to register ring fd");
        }

        self.enter_fd = unsafe { BorrowedFd::borrow_raw(args.offset as _) };
        self.enter_flags |= IoUringEnterFlags::REGISTERED_RING;
        Ok(NULL)
    }

    // Unsafe: enter_fd must registered and unregister in drop() function
    pub(crate) unsafe fn unregister_ring_fd(&mut self) -> Result<u32> {
        let idx = self.enter_fd.as_raw_fd().cast_unsigned();
        let args = IoUringRsrcUpdate::unregister(idx);
        Ok(unsafe { io_uring_register(self.enter_fd, UnregisterRingFds, args.as_ptr(), 1)? })
    }
}
