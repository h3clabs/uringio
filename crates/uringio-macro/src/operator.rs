pub mod attr;
pub mod item;
pub mod setter;

use syn::__private::{
    TokenStream2,
    quote::{ToTokens, quote},
};

use crate::operator::{attr::Attr, item::Item};

#[derive(Debug)]
pub struct Operator {
    pub attr: Attr,
    pub item: Item,
}

impl Operator {
    pub const fn new(attr: Attr, item: Item) -> Self {
        Self { attr, item }
    }

    pub fn gen_code(&self) -> TokenStream2 {
        let name = &self.item.ident;
        let generics = self.item.generic_quote();

        let impl_op = self.attr.gen_impl_trait_op(name, &generics);
        let impl_setters = self.item.gen_fn_setter_methods();
        let test_size_align = self.attr.gen_test_size_align(name);

        quote! {
            #impl_op
            #impl_setters
            #test_size_align
        }
    }
}

impl ToTokens for Operator {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self { item, .. } = self;
        let gen_code = self.gen_code();

        let token = quote! {
            #item
            #gen_code
        };
        tokens.extend(token);
    }
}
