use std::{
    marker::PhantomData,
    mem::transmute,
    ops::{Deref, DerefMut},
};

use crate::platform::iouring::{IoUringCqe, IoUringCqeFlags, IoUringSetupFlags};

#[derive(Debug)]
pub enum Ty {
    Cqe16,
    Cqe32,
    CqeMix,
}

const BASE_CQE_SIZE: usize = size_of::<IoUringCqe>();

pub trait Cqe {
    const TYPE: Ty;

    const SETUP_FLAG: IoUringSetupFlags = match Self::TYPE {
        Ty::Cqe16 => IoUringSetupFlags::empty(),
        Ty::Cqe32 => IoUringSetupFlags::CQE32,
        Ty::CqeMix => IoUringSetupFlags::CQE_MIXED,
    };

    const SETUP_CQE_SIZE: usize = match Self::TYPE {
        Ty::Cqe16 | Ty::CqeMix => BASE_CQE_SIZE,
        Ty::Cqe32 => BASE_CQE_SIZE * 2,
    };
}

pub trait FixCqe: Sized {}

/// ## Cqe16
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct Cqe16 {
    raw: IoUringCqe,
}

impl Cqe for Cqe16 {
    const TYPE: Ty = Ty::Cqe16;
}

impl FixCqe for Cqe16 {}

impl Deref for Cqe16 {
    type Target = IoUringCqe;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl DerefMut for Cqe16 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

/// ## Cqe32
#[derive(Debug, Default)]
#[repr(C)]
pub struct Cqe32 {
    raw: IoUringCqe,
    ext_data: [u64; 2],
}

impl Cqe for Cqe32 {
    const TYPE: Ty = Ty::Cqe32;
}

impl FixCqe for Cqe32 {}

impl Cqe32 {
    pub const fn ext_data(&self) -> &[u64; 2] {
        &self.ext_data
    }
}

impl Deref for Cqe32 {
    type Target = IoUringCqe;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl DerefMut for Cqe32 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

/// ## Cqe Mixed
#[derive(Debug, Default)]
#[repr(C)]
pub struct CqeMix {
    raw: IoUringCqe,
    ext_data: PhantomData<[u64; 2]>,
}

impl Cqe for CqeMix {
    const TYPE: Ty = Ty::CqeMix;
}

impl CqeMix {
    #[inline]
    pub fn is_cqe32(&self) -> bool {
        self.flags.contains(IoUringCqeFlags::CQE_32)
    }

    pub const unsafe fn ext_data(&self) -> &[u64; 2] {
        unsafe { transmute(&self.ext_data) }
    }
}

impl<T> From<T> for CqeMix
where
    T: Into<Cqe16>,
{
    fn from(cqe: T) -> Self {
        Self { raw: cqe.into().raw, ext_data: PhantomData }
    }
}

impl Deref for CqeMix {
    type Target = IoUringCqe;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl DerefMut for CqeMix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cqe_size() {
        assert_eq!(Cqe16::SETUP_CQE_SIZE, 16);
        assert_eq!(Cqe32::SETUP_CQE_SIZE, 32);
        assert_eq!(CqeMix::SETUP_CQE_SIZE, 16);
    }

    #[test]
    fn test_cqe_align() {
        assert_eq!(align_of::<Cqe16>(), 8);
        assert_eq!(align_of::<Cqe32>(), 8);
        assert_eq!(align_of::<CqeMix>(), 8);
    }

    #[test]
    fn test_entry_size() {
        assert_eq!(size_of::<Cqe16>(), 16);
        assert_eq!(size_of::<Cqe32>(), 32);
        assert_eq!(size_of::<CqeMix>(), 16);
    }

    #[test]
    fn test_cqe_mix_transmute() {
        assert_eq!(size_of::<Cqe16>(), size_of::<CqeMix>());
        assert_eq!(align_of::<Cqe16>(), align_of::<CqeMix>());
    }
}
