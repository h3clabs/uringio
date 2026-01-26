pub use rustix::{
    fd::{AsFd, AsRawFd, BorrowedFd, OwnedFd, RawFd},
    ffi::c_void,
    io::{ReadWriteFlags, Result},
    io_uring::{
        IORING_OFF_CQ_RING as IOURING_OFF_CQ_RING, IORING_OFF_SQ_RING as IOURING_OFF_SQ_RING,
        IORING_OFF_SQES as IOURING_OFF_SQES, IoringCqFlags as IoUringCqFlags,
        IoringCqeFlags as IoUringCqeFlags, IoringEnterFlags as IoUringEnterFlags,
        IoringFeatureFlags as IoUringFeatureFlags, IoringOp as IoUringOp,
        IoringRegisterOp as IoUringRegisterOp, IoringSetupFlags as IoUringSetupFlags,
        IoringSqFlags as IoUringSqFlags, IoringSqeFlags as IoUringSqeFlags,
        io_uring_cqe as IoUringCqe, io_uring_enter, io_uring_params as IoUringParams,
        io_uring_ptr as IoUringPtr, io_uring_register, io_uring_rsrc_update as IoUringRsrcUpdate,
        io_uring_setup, io_uring_sqe as IoUringSqe, io_uring_user_data as IoUringUserData,
    },
};

pub const IOURING_MAX_SQ_ENTRIES: u32 = 1 << 15;

pub const IOURING_MAX_CQ_ENTRIES: u32 = IOURING_MAX_SQ_ENTRIES * 2;

pub const IOURING_IO_RINGS_SIZE: usize = 64; // size_of::<struct io_rings {...}>()

// TODO: patch to rustix
#[derive(Debug, Copy, Clone, Default)]
#[repr(C)]
pub struct IoUringPiAttr {
    pub ptr: IoUringPtr,
    pub mask: u64, // TODO: IORING_RW_ATTR_FLAG_PI (1U << 0)
}

#[derive(Debug, Copy, Clone, Default)]
#[repr(C)]
pub struct IoUringWritePi {
    pub flags: u16,
    pub app_tag: u16,
    pub len: u32,
    pub addr: u64,
    pub seed: u64,
    pub rsvd: u64,
}

// TODO: bit flags
#[derive(Debug, Copy, Clone, Default)]
pub struct NopFlags {}

#[rustfmt::skip]
impl NopFlags {
    // Default
    pub const NONE: u32 = 0;

    // IORING_NOP_INJECT_RESULT
    pub const INJECT_RESULT: u32 = 1 << 0;

    // IORING_NOP_FILE
    pub const FILE: u32 = 1 << 1;

    // IORING_NOP_FIXED_FILE
    pub const FIXED_FILE: u32 = 1 << 2;

    // IORING_NOP_FIXED_BUFFER
    pub const FIXED_BUFFER: u32 = 1 << 3;

    // IORING_NOP_TW
    pub const TW: u32 = 1 << 4;

    // IORING_NOP_CQE32
    pub const CQE32: u32 = 1 << 5;
}
