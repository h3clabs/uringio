use crate::platform::iouring::{IoUringRsrcUpdate, IoUringUserData, RawFd, c_void};

pub trait RegisterArgs {
    fn as_ptr(&self) -> *const c_void;
}

impl RegisterArgs for IoUringRsrcUpdate {
    fn as_ptr(&self) -> *const c_void {
        (&raw const *self).cast()
    }
}

pub trait RegisterRingFd {
    fn new(fd: RawFd) -> Self;

    fn unregister(idx: u32) -> Self;
}

impl RegisterRingFd for IoUringRsrcUpdate {
    fn new(fd: RawFd) -> Self {
        let mut this = Self::default();
        this.offset = u32::MAX; // -1U
        this.data = unsafe { IoUringUserData::from(fd as u64).ptr };
        this
    }

    fn unregister(idx: u32) -> Self {
        let mut this = Self::default();
        this.offset = idx;
        this
    }
}
