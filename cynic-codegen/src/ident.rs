use inflector::Inflector;
use proc_macro2::TokenStream;

/// A convenience type for working with identifiers we write out in our macros.
#[derive(Debug, Clone)]
pub struct Ident(String);

impl Ident {
    pub fn for_inbuilt_scalar(s: &str) -> Self {
        Ident(s.to_string())
    }

    pub fn for_type(s: &str) -> Self {
        Ident(s.to_pascal_case())
    }

    pub fn for_field(s: &str) -> Self {
        Ident(s.to_snake_case())
    }
}

impl Into<proc_macro2::Ident> for &Ident {
    fn into(self) -> proc_macro2::Ident {
        use quote::format_ident;
        if self.0 == "type" {
            format_ident!("{}_", self.0)
        } else {
            format_ident!("{}", self.0)
        }
    }
}

impl quote::ToTokens for Ident {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let macro_ident: proc_macro2::Ident = self.into();
        macro_ident.to_tokens(tokens);
    }
}
