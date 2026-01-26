use std::io::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

macro_rules! err {
    ($($arg:tt)*) => {
        Err(std::io::Error::new(std::io::ErrorKind::Other, format!($($arg)*)))
    };
}

pub(crate) use err;
