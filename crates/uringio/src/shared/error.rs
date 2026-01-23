use std::io::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

macro_rules! err {
    ($expr:expr) => {
        Err(std::io::Error::new(std::io::ErrorKind::Other, $expr))
    };
}

pub(crate) use err;
