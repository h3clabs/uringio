use std::ops::Deref;

use syn::{
    __private::{
        TokenStream2,
        quote::{ToTokens, quote},
    },
    Fields, GenericParam, ItemStruct, Result,
    parse::{Parse, ParseStream},
};

use crate::operator::setter::parse_setter_fields;

#[derive(Debug)]
pub struct Item {
    inner: ItemStruct,
}

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        let inner = input.parse()?;
        Ok(Self { inner })
    }
}

impl Item {
    pub fn generic_params(&self) -> Vec<TokenStream2> {
        self.inner
            .generics
            .params
            .iter()
            .map(|param| {
                match param {
                    GenericParam::Lifetime(_) => quote!('_),
                    GenericParam::Type(tp) => {
                        let ident = &tp.ident;
                        quote!(#ident)
                    },
                    GenericParam::Const(cp) => {
                        let ident = &cp.ident;
                        quote!(#ident)
                    },
                }
            })
            .collect()
    }

    pub fn generic_quote(&self) -> TokenStream2 {
        let generics = self.generic_params();
        if generics.is_empty() {
            quote! {}
        } else {
            quote! { <#(#generics),*> }
        }
    }

    pub fn gen_fn_setter_methods(&self) -> TokenStream2 {
        let name = &self.inner.ident;
        let generics = self.generic_quote();
        let setters = parse_setter_fields(&self.inner.fields);

        if setters.is_empty() {
            return quote! {};
        }

        quote! {
            impl #name #generics {
                #(#setters)*
            }
        }
    }
}

impl ToTokens for Item {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut item = self.inner.clone();

        // Strip #[setter(...)] attributes from fields
        if let Fields::Named(fields) = &mut item.fields {
            for field in &mut fields.named {
                field.attrs.retain(|attr| !attr.path().is_ident("setter"));
            }
        }

        item.to_tokens(tokens);
    }
}

impl Deref for Item {
    type Target = ItemStruct;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
