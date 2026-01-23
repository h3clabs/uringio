use std::{
    fs::{File, OpenOptions},
    io,
    os::unix::{fs::OpenOptionsExt, io::AsRawFd},
};

use uringio::{
    operator::{noop::Nop, read::Read},
    platform::iouring::IoUringUserData,
    shared::{
        error::Result,
        log::{info, init_tracing_subscriber_log, trace},
        null::{NULL, Null},
    },
    submission::{
        entry::{Sqe64, Sqe128},
        submitter::Submit,
    },
    uring::{
        Uring128, UringIo, UringMix,
        enter::UringEnter,
        mode::{Iopoll, Sqpoll},
    },
};

#[test]
pub fn readme() -> Result<Null> {
    init_tracing_subscriber_log();

    let (fd, args, arena) = Sqpoll::new(128).no_mmap().setup()?;
    let mut uring = UringIo::new(&fd, &args, arena)?.register()?;

    info!("uring: {:?}", uring);

    let file = File::open("../../README.md")?;

    let mut dst = vec![0; 1024];
    let mut read = Read::new(&file, &mut dst);
    read.user_data = IoUringUserData::from(0x42);
    info!("Read: {:?}", read);

    let (enter, mut submitter, mut collector) = uring.borrow();

    if let Err(sqe) = submitter.push(read) {
        panic!("submission queue is full");
    }

    loop {
        info!("rerun");

        submitter.submit();

        collector.update();
        if let Some(cqe) = collector.next() {
            trace!("== cqe ==: {:?}", cqe);
            trace!("== dst ==: {:?}", str::from_utf8(&dst).unwrap());
            break;
        } else {
            println!("flush collector");
            collector.flush(enter, 1)?;
        }

        // std::thread::sleep(std::time::Duration::from_millis(1500));
    }

    Ok(NULL)
}
