use std::marker::PhantomData;

use crate::{
    operator::{Op, fd::OpFd},
    platform::iouring::{
        IoUringOp, IoUringPiAttr, IoUringPtr, IoUringSqeFlags, IoUringUserData, RawFd,
        ReadWriteFlags,
    },
    shared::macros::op,
    submission::entry::Sqe64,
};

#[derive(Debug)]
#[op(Read, Entry = Sqe64)]
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
    #[setter]
    pub user_data: IoUringUserData,
    _unused0_: [u8; 2],
    pub personality: u16,
    _unused1_: [u8; 4],
    pub pi_attr: IoUringPiAttr,

    _marker_: PhantomData<(&'fd (), &'dst mut [u8])>,
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
