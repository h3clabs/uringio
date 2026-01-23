use crate::platform::iouring::IoUringOp;

pub trait Opcode {
    fn is_sqe128(&self) -> bool;
}

impl Opcode for IoUringOp {
    #[inline]
    fn is_sqe128(&self) -> bool {
        matches!(self, IoUringOp::Nop128 | IoUringOp::UringCmd128)
    }
}
