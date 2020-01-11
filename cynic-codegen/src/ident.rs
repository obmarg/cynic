use inflector::Inflector;
use proc_macro2::TokenStream;

/// A convenience type for working with identifiers we write out in our macros.
#[derive(Debug, Clone)]
pub struct Ident(String);

impl Ident {
    pub fn for_inbuilt_scalar(s: &str) -> Self {
        Ident(transform_keywords(s.to_string()))
    }

    pub fn for_type(s: &str) -> Self {
        Ident(transform_keywords(s.to_pascal_case()))
    }

    pub fn for_field(s: &str) -> Self {
        Ident(transform_keywords(s.to_snake_case()))
    }

    pub fn for_module(s: &str) -> Self {
        let ident = s.to_snake_case();
        if ident == "super" {
            // This is an allowed keyword for modules.
            Ident(ident)
        } else {
            Ident(transform_keywords(ident))
        }
    }
}

fn transform_keywords(mut s: String) -> String {
    match syn::parse_str::<syn::Ident>(&s) {
        Ok(_) => s,
        Err(_) => {
            // s is _probably_ a keyword
            s.push('_');
            s
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_keywords() {
        assert_eq!(transform_keywords("test".to_string()), "test".to_string());
        assert_eq!(transform_keywords("type".to_string()), "type_".to_string());
    }
}
