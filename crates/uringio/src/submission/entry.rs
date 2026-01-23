use std::{
    any::type_name,
    fmt::{Debug, Formatter},
    marker::PhantomData,
    mem::transmute,
    ops::{Deref, DerefMut},
};

use crate::{
    operator::opcode::Opcode,
    platform::iouring::{IoUringSetupFlags, IoUringSqe},
};

#[derive(Debug)]
pub enum Ty {
    Sqe64,
    Sqe128,
    SqeMix,
}

const BASE_SQE_SIZE: usize = size_of::<IoUringSqe>();

pub trait Sqe {
    const TYPE: Ty;

    const SETUP_FLAG: IoUringSetupFlags = match Self::TYPE {
        Ty::Sqe64 => IoUringSetupFlags::empty(),
        Ty::Sqe128 => IoUringSetupFlags::SQE128,
        Ty::SqeMix => IoUringSetupFlags::SQE_MIXED,
    };

    const SETUP_SQE_SIZE: usize = match Self::TYPE {
        Ty::Sqe64 | Ty::SqeMix => BASE_SQE_SIZE,
        Ty::Sqe128 => BASE_SQE_SIZE * 2,
    };
}

pub trait FixSqe: Sized {}

/// ## Sqe64
#[repr(transparent)]
pub struct Sqe64 {
    raw: IoUringSqe,
}

impl Sqe for Sqe64 {
    const TYPE: Ty = Ty::Sqe64;
}

impl FixSqe for Sqe64 {}

impl Debug for Sqe64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: struct detail
        f.debug_struct(type_name::<Self>()).finish()
    }
}

impl Sqe64 {
    pub const fn new(sqe: IoUringSqe) -> Self {
        Self { raw: sqe }
    }
}

impl Deref for Sqe64 {
    type Target = IoUringSqe;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl DerefMut for Sqe64 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

/// ## Sqe128
#[repr(C)]
pub struct Sqe128 {
    raw: IoUringSqe,
    ext_data: [u8; 64],
}

impl Sqe for Sqe128 {
    const TYPE: Ty = Ty::Sqe128;
}

impl FixSqe for Sqe128 {}

impl Debug for Sqe128 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: struct detail
        f.debug_struct(type_name::<Self>()).finish()
    }
}

impl Sqe128 {
    pub const fn new(raw: IoUringSqe) -> Self {
        Self { raw, ext_data: [0; 64] }
    }

    #[inline]
    pub fn uring_cmd(&mut self) -> &mut [u8; 80] {
        unsafe { transmute(&mut self.addr3_or_cmd.cmd) }
    }
}

impl Deref for Sqe128 {
    type Target = IoUringSqe;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl DerefMut for Sqe128 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

/// ## Sqe Mixed
#[repr(transparent)]
pub struct SqeMix {
    raw: IoUringSqe,
    extra_data: PhantomData<[u8; 64]>,
}

impl Sqe for SqeMix {
    const TYPE: Ty = Ty::SqeMix;
}

impl Debug for SqeMix {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: struct detail
        f.debug_struct(type_name::<Self>()).finish()
    }
}

impl SqeMix {
    #[inline]
    pub fn is_sqe128(&self) -> bool {
        self.opcode.is_sqe128()
    }

    #[inline]
    pub unsafe fn uring_cmd(&mut self) -> &mut [u8; 80] {
        unsafe { transmute(&mut self.addr3_or_cmd.cmd) }
    }
}

impl<T> From<T> for SqeMix
where
    T: Into<Sqe64>,
{
    fn from(sqe: T) -> Self {
        Self { raw: sqe.into().raw, extra_data: PhantomData }
    }
}

impl Deref for SqeMix {
    type Target = IoUringSqe;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl DerefMut for SqeMix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqe_size() {
        assert_eq!(Sqe64::SETUP_SQE_SIZE, 64);
        assert_eq!(Sqe128::SETUP_SQE_SIZE, 128);
        assert_eq!(SqeMix::SETUP_SQE_SIZE, 64);
    }

    #[test]
    fn test_sqe_align() {
        assert_eq!(align_of::<Sqe64>(), 8);
        assert_eq!(align_of::<Sqe128>(), 8);
        assert_eq!(align_of::<SqeMix>(), 8);
    }

    #[test]
    fn test_entry_size() {
        assert_eq!(size_of::<Sqe64>(), 64);
        assert_eq!(size_of::<Sqe128>(), 128);
        assert_eq!(size_of::<SqeMix>(), 64);
    }

    #[test]
    fn test_sqe_mix_transmute() {
        assert_eq!(size_of::<Sqe64>(), size_of::<SqeMix>());
        assert_eq!(align_of::<Sqe64>(), align_of::<SqeMix>());
    }
}
