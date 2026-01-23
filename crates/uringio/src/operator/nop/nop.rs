use std::marker::PhantomData;

use crate::{
    operator::{Op, fd::OpFd},
    platform::iouring::{IoUringOp, IoUringSqeFlags, IoUringUserData, NopFlags, RawFd},
    submission::entry::Sqe64,
};

#[derive(Debug)]
#[repr(C)]
pub struct Nop<'fd> {
    pub opcode: IoUringOp,
    pub flags: IoUringSqeFlags,
    _unused0_: [u8; 2],
    pub fd: RawFd,
    _unused1_: u64,
    _unused2_: u64,
    pub len: u32,
    pub nop_flags: u32, // TODO: NopFlags
    pub user_data: IoUringUserData,
    pub buf_index: u16,
    _unused3_: [u8; 2],
    _unused4_: [u8; 4],
    _unused5_: [u8; 16],

    _marker_: PhantomData<&'fd RawFd>,
}

impl Op for Nop<'_> {
    type Entry = Sqe64;

    const OP_CODE: IoUringOp = IoUringOp::Nop;
}

impl<'fd> Nop<'fd> {
    pub fn new() -> Self {
        Self {
            opcode: Self::OP_CODE,
            flags: IoUringSqeFlags::default(),
            _unused0_: Default::default(),
            fd: -1,
            _unused1_: 0,
            _unused2_: 0,
            len: 0,
            nop_flags: NopFlags::NONE,
            user_data: Default::default(),
            buf_index: Default::default(),
            _unused3_: Default::default(),
            _unused4_: Default::default(),
            _unused5_: Default::default(),
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
        Nop::check_size_align();
    }
}
