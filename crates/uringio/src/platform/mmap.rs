use std::{
    fmt::Debug,
    ptr::{NonNull, null_mut},
};

pub use rustix::{
    fd::AsFd,
    ffi::c_void,
    mm::{MapFlags, ProtFlags, mmap, mmap_anonymous, munmap},
    param::page_size,
};

use crate::shared::{
    error::Result,
    log::{Level, instrument},
};

/// ## Mmap pointer
pub type Ptr = NonNull<c_void>;

/// ## Mmap
#[derive(Debug)]
pub struct Mmap {
    ptr: Ptr,
    len: usize,
}

impl Mmap {
    const MAP_FLAG: MapFlags = MapFlags::SHARED.union(MapFlags::POPULATE);
    const MAP_PROT: ProtFlags = ProtFlags::READ.union(ProtFlags::WRITE);

    #[instrument(level = Level::TRACE, ret, err)]
    pub fn new<Fd>(fd: Fd, len: usize, offset: u64) -> Result<Self>
    where
        Fd: Debug + AsFd,
    {
        let ptr = unsafe {
            let mem = mmap(null_mut(), len, Self::MAP_PROT, Self::MAP_FLAG, fd, offset)?;
            Ptr::new_unchecked(mem)
        };
        Ok(Self { ptr, len })
    }

    #[instrument(level = Level::TRACE, ret, err)]
    pub unsafe fn mmap<Fd>(
        ptr: *mut c_void,
        len: usize,
        prot: ProtFlags,
        flags: MapFlags,
        fd: Fd,
        offset: u64,
    ) -> Result<Self>
    where
        Fd: Debug + AsFd,
    {
        let ptr = unsafe {
            let mem = mmap(ptr, len, prot, flags, fd, offset)?;
            Ptr::new_unchecked(mem)
        };
        Ok(Self { ptr, len })
    }

    #[instrument(level = Level::TRACE, ret, err)]
    pub unsafe fn mmap_anonymous(
        ptr: *mut c_void,
        len: usize,
        prot: ProtFlags,
        flags: MapFlags,
    ) -> Result<Self> {
        let ptr = unsafe {
            let mem = mmap_anonymous(ptr, len, prot, flags)?;
            Ptr::new_unchecked(mem)
        };
        Ok(Self { ptr, len })
    }

    #[inline]
    pub const fn ptr(&self) -> Ptr {
        self.ptr
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }
}

impl Drop for Mmap {
    #[instrument(level = Level::TRACE)]
    fn drop(&mut self) {
        // TODO: catch error
        unsafe {
            let _ = munmap(self.ptr.as_ptr(), self.len);
        };
    }
}

#[inline]
pub fn page_align(size: usize) -> usize {
    let page_size = page_size();
    (size + page_size - 1) & !(page_size - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_size() {
        assert!(page_size().is_power_of_two());
    }
}
