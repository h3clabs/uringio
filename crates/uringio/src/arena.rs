pub mod huge;
pub mod mmap;

use crate::platform::mmap::Ptr;

pub trait Arena<M, S, C>: Sized {
    fn sq(&self) -> Ptr;

    fn sqes(&self) -> Ptr;

    fn cq(&self) -> Ptr;
}
