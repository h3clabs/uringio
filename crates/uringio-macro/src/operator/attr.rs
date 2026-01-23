use syn::{
    __private::{TokenStream2, quote::quote},
    Ident, Result, Token,
    parse::{Parse, ParseStream},
};

mod args {
    syn::custom_keyword!(Entry);
}

#[derive(Debug)]
pub struct Attr {
    pub code: Ident,
    pub entry: Ident,
}

impl Parse for Attr {
    fn parse(input: ParseStream) -> Result<Self> {
        let code = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;
        input.parse::<args::Entry>()?;
        input.parse::<Token![=]>()?;
        let entry = input.parse::<Ident>()?;

        Ok(Self { code, entry })
    }
}

impl Attr {
    pub fn gen_impl_trait_op(&self, name: &Ident, generics: &TokenStream2) -> TokenStream2 {
        let code = &self.code;
        let entry = &self.entry;

        quote! {
            impl Op for #name #generics {
                type Entry = #entry;
                const OP_CODE: IoUringOp = IoUringOp::#code;
            }
        }
    }

    pub fn gen_test_size_align(&self, op: &Ident) -> TokenStream2 {
        quote! {
            #[cfg(test)]
            mod _gen_op_tests_ {
                use super::*;

                #[test]
                fn test_size_align() {
                    #op::check_size_align();
                }
            }
        }
    }
}
