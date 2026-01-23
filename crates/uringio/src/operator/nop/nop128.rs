use std::marker::PhantomData;

use crate::{
    operator::{Op, fd::OpFd},
    platform::iouring::{IoUringOp, IoUringSqeFlags, IoUringUserData, NopFlags, RawFd},
    submission::entry::Sqe64,
};

#[derive(Debug)]
#[repr(C)]
pub struct Nop128<'fd> {
    pub opcode: IoUringOp,
    pub flags: IoUringSqeFlags,
    _unused0_: [u8; 2],
    pub fd: RawFd,
    pub ext_data1: u64,
    pub ext_data2: u64,
    pub len: u32,
    pub nop_flags: u32, // TODO: NopFlags
    pub user_data: IoUringUserData,
    pub buf_index: u16,
    _unused1_: [u8; 2],
    _unused2_: [u8; 4],
    _unused3_: [u8; 16],

    _marker_: PhantomData<&'fd RawFd>,
}

impl Op for Nop128<'_> {
    type Entry = Sqe64;

    const OP_CODE: IoUringOp = IoUringOp::Nop128;
}

impl<'fd> Nop128<'fd> {
    pub fn new() -> Self {
        Self {
            opcode: Self::OP_CODE,
            flags: IoUringSqeFlags::default(),
            _unused0_: Default::default(),
            fd: -1,
            ext_data1: 0,
            ext_data2: 0,
            len: 0,
            nop_flags: NopFlags::NONE,
            user_data: Default::default(),
            buf_index: Default::default(),
            _unused1_: Default::default(),
            _unused2_: Default::default(),
            _unused3_: Default::default(),
            _marker_: PhantomData,
        }
    }

    pub fn set_fd<Fd>(mut self, fd: &'fd Fd) -> Self
    where
        Fd: OpFd,
    {
        self.fd = fd.raw_fd();
        self.nop_flags |= Fd::NOP_FLAG;
        self
    }

    pub fn set_buf_index(mut self, buf_index: u16) -> Self {
        self.buf_index = buf_index;
        self.nop_flags |= NopFlags::FIXED_BUFFER;
        self
    }

    pub fn set_ext_data(mut self, ext_data: [u64; 2]) -> Self {
        self.ext_data1 = ext_data[0];
        self.ext_data2 = ext_data[1];
        self.nop_flags |= NopFlags::CQE32;
        self
    }

    pub fn skip_cqe(mut self) -> Self {
        self.flags |= IoUringSqeFlags::CQE_SKIP_SUCCESS;
        self
    }

    pub fn enable_tw(mut self) -> Self {
        self.nop_flags |= NopFlags::TW;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_align() {
        Nop128::check_size_align();
    }
}
