use std::marker::PhantomData;

use crate::{
    operator::{Op, fd::OpFd},
    platform::{
        io::ReadWriteFlags,
        iouring::{IoUringOp, IoUringPiAttr, IoUringPtr, IoUringSqeFlags, IoUringUserData, RawFd},
    },
    submission::entry::Sqe64,
};

#[derive(Debug)]
#[repr(C)]
pub struct Read<'fd, 'dst> {
    pub opcode: IoUringOp,
    pub flags: IoUringSqeFlags,
    pub ioprio: u16,
    pub fd: RawFd,
    pub offset: u64,
    pub ptr: IoUringPtr,
    pub len: u32,
    pub rw_flags: ReadWriteFlags,
    pub user_data: IoUringUserData,
    _unused0_: [u8; 2],
    pub personality: u16,
    _unused1_: [u8; 4],
    pub pi_attr: IoUringPiAttr,

    _marker_: PhantomData<(&'fd (), &'dst mut [u8])>,
}

impl Op for Read<'_, '_> {
    type Entry = Sqe64;

    const OP_CODE: IoUringOp = IoUringOp::Read;
}

impl<'fd, 'dst> Read<'fd, 'dst> {
    pub fn new<Fd>(fd: &'fd Fd, dst: &'dst mut [u8]) -> Self
    where
        Fd: OpFd,
    {
        Self {
            opcode: Self::OP_CODE,
            flags: Fd::SQE_FLAG,
            ioprio: 0,
            fd: fd.raw_fd(),
            offset: 0,
            ptr: IoUringPtr::new(dst.as_mut_ptr().cast()),
            len: dst.len() as _,
            rw_flags: Default::default(),
            user_data: Default::default(),
            _unused0_: Default::default(),
            personality: Default::default(),
            _unused1_: Default::default(),
            pi_attr: Default::default(),
            _marker_: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_align() {
        Read::check_size_align();
    }
}
