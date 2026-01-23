macro_rules! err {
    ($($arg:tt)*) => {
        Err(::syn::Error::new(::syn::__private::Span::call_site(), format!($($arg)*)))
    }
}

#[allow(unused_imports)]
pub(crate) use err;
