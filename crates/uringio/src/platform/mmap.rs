use std::ptr::{NonNull, null_mut};

use rustix::{
    fd::AsFd,
    ffi::c_void,
    mm::{MapFlags, ProtFlags, mmap, munmap},
};

use crate::shared::error::Result;

/// ## Mmap pointer
pub type Ptr = NonNull<c_void>;

/// ## Mmap
#[derive(Debug)]
pub struct Mmap {
    ptr: Ptr,
    len: usize,
}

impl Drop for Mmap {
    fn drop(&mut self) {
        // TODO: catch error
        unsafe {
            let _ = munmap(self.ptr.as_ptr(), self.len);
        };
    }
}

impl Mmap {
    const MAP_FLAG: MapFlags = MapFlags::SHARED.union(MapFlags::POPULATE);
    const MAP_PROT: ProtFlags = ProtFlags::READ.union(ProtFlags::WRITE);

    pub fn new<Fd>(fd: Fd, len: usize, offset: u64) -> Result<Self>
    where
        Fd: AsFd,
    {
        let ptr = unsafe {
            let mem = mmap(null_mut(), len, Self::MAP_PROT, Self::MAP_FLAG, fd, offset)?;
            Ptr::new_unchecked(mem)
        };
        Ok(Self { ptr, len })
    }

    pub unsafe fn map<Fd>(
        ptr: *mut c_void,
        len: usize,
        prot: ProtFlags,
        flags: MapFlags,
        fd: Fd,
        offset: u64,
    ) -> Result<Self>
    where
        Fd: AsFd,
    {
        let ptr = unsafe {
            let mem = mmap(ptr, len, prot, flags, fd, offset)?;
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

    #[inline]
    pub const unsafe fn offset(&self, offset: u32) -> Ptr {
        unsafe { self.ptr.byte_add(offset as usize) }
    }
}
