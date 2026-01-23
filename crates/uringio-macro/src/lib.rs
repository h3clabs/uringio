mod error;
mod operator;

use proc_macro::TokenStream;
use syn::{
    __private::{TokenStream2, quote::quote},
    Result, parse,
};

use crate::operator::{Operator, attr::Attr, item::Item};

#[proc_macro_attribute]
pub fn op(attr: TokenStream, item: TokenStream) -> TokenStream {
    match op_impl(attr, item) {
        Ok(token) => token,
        Err(err) => err.into_compile_error(),
    }
    .into()
}

fn op_impl(attr: TokenStream, item: TokenStream) -> Result<TokenStream2> {
    let attr = parse::<Attr>(attr)?;
    let item = parse::<Item>(item)?;
    let operator = Operator::new(attr, item);
    Ok(quote!(#operator))
}
