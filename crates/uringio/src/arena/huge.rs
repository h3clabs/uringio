use std::{marker::PhantomData, ptr::null_mut};

use crate::{
    arena::Arena,
    completion::entry::Cqe,
    platform::{
        iouring::{
            IOURING_IO_RINGS_SIZE, IOURING_MAX_CQ_ENTRIES, IOURING_MAX_SQ_ENTRIES,
            IoUringSetupFlags,
        },
        mmap::{MapFlags, Mmap, ProtFlags, Ptr, page_align, page_size},
    },
    shared::{
        error::{Result, err},
        log::debug,
    },
    submission::entry::Sqe,
    uring::{args::UringArgs, mode::Mode},
};

// TODO: use platform huge page size
pub const HUGE_PAGE_SIZE: usize = 2 * 1024 * 1024; // 2MiB

/// ## Huge Arena
#[derive(Debug)]
pub struct HugeArena<M, S, C> {
    pub ring_mmap: Mmap,
    pub sqes_addr: Ptr,
    pub sqes_mmap: Option<Mmap>,

    _marker_: PhantomData<(M, S, C)>,
}

impl<M, S, C> HugeArena<M, S, C>
where
    M: Mode,
    S: Sqe,
    C: Cqe,
{
    pub fn setup(args: &mut UringArgs<Self, M, S, C>) -> Result<Self> {
        debug_assert!(args.sq_entries > 0);
        debug_assert!(args.flags.contains(IoUringSetupFlags::CLAMP));
        debug_assert!(args.flags.contains(IoUringSetupFlags::NO_MMAP));

        let sqsize = args.sq_entries.min(IOURING_MAX_SQ_ENTRIES).next_power_of_two();
        let cqsize = if args.flags.contains(IoUringSetupFlags::CQSIZE) {
            debug_assert!(args.cq_entries > 0);
            args.cq_entries.min(IOURING_MAX_CQ_ENTRIES).next_power_of_two()
        } else {
            args.params.flags |= IoUringSetupFlags::CQSIZE;
            sqsize * 2
        };
        // set sqsize and cqsize
        args.params.sq_entries = sqsize;
        args.params.cq_entries = cqsize;

        let ring_mem = IOURING_IO_RINGS_SIZE + args.cqes_mem() + args.sq_indices_mem();
        let sqes_mem = args.sqes_mem();
        debug!("ring_mem: {ring_mem}, sqes_mem: {sqes_mem}");

        // page align ring and sqes mem size
        let ring_size = page_align(ring_mem);
        let sqes_size = page_align(sqes_mem);
        debug!("ring_size: {ring_size}, sqes_size: {sqes_size}");

        if ring_size > HUGE_PAGE_SIZE || sqes_size > HUGE_PAGE_SIZE {
            return err!("IoUring mem exceeds HugePage size: {HUGE_PAGE_SIZE}");
        }

        // mmap ring and sqes mem
        let ring_mmap = Self::mmap(ring_size)?;
        let sqes_mmap = if ring_mmap.len() < ring_size + sqes_size {
            let sqes_mmap = Self::mmap(sqes_size)?;
            Some(sqes_mmap)
        } else {
            debug!("sqes use shared ring mmap");
            None
        };

        // set sq and cq user addr
        let sqes_addr = match &sqes_mmap {
            Some(sqes_mmap) => sqes_mmap.ptr(),
            None => unsafe { ring_mmap.ptr().byte_add(ring_size) },
        };
        args.params.cq_off.user_addr = ring_mmap.ptr().as_ptr().into();
        args.params.sq_off.user_addr = sqes_addr.as_ptr().into();

        Ok(Self { ring_mmap, sqes_mmap, sqes_addr, _marker_: PhantomData })
    }

    pub fn mmap(size: usize) -> Result<Mmap> {
        let (size, flags) = if size <= page_size() {
            (size, MapFlags::SHARED)
        } else {
            (HUGE_PAGE_SIZE, MapFlags::SHARED | MapFlags::HUGETLB)
        };

        unsafe { Mmap::mmap_anonymous(null_mut(), size, ProtFlags::READ | ProtFlags::WRITE, flags) }
    }
}

impl<M, S, C> Arena<M, S, C> for HugeArena<M, S, C>
where
    M: Mode,
    S: Sqe,
    C: Cqe,
{
    fn sq(&self) -> Ptr {
        self.ring_mmap.ptr()
    }

    fn sqes(&self) -> Ptr {
        self.sqes_addr
    }

    fn cq(&self) -> Ptr {
        self.ring_mmap.ptr()
    }
}
