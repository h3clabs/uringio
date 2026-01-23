use syn::{
    __private::{
        TokenStream2,
        quote::{ToTokens, quote},
    },
    Fields, Ident, Type,
};

#[derive(Debug)]
pub struct Setter {
    pub fn_name: Ident,
    pub field_name: Ident,
    pub field_type: Type,
}

impl Setter {
    const IDENT: &str = "setter";
}

impl ToTokens for Setter {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let fn_name = &self.fn_name;
        let field_name = &self.field_name;
        let field_type = &self.field_type;

        let token = quote! {
            pub fn #fn_name<T: Into<#field_type>>(mut self, value: T) -> Self {
                self.#field_name = value.into();
                self
            }
        };
        tokens.extend(token);
    }
}

pub fn parse_setter_fields(fields: &Fields) -> Vec<Setter> {
    let mut setters = Vec::new();

    if let Fields::Named(fields) = fields {
        for field in &fields.named {
            for attr in &field.attrs {
                if attr.path().is_ident(Setter::IDENT) {
                    if let Some(field_name) = &field.ident {
                        let fn_name = match attr.parse_args::<Ident>() {
                            Ok(ident) => ident,
                            Err(_) => field_name.clone(),
                        };

                        setters.push(Setter {
                            fn_name,
                            field_name: field_name.clone(),
                            field_type: field.ty.clone(),
                        });
                    }
                }
            }
        }
    }

    setters
}
