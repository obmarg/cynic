use inflector::Inflector;
use proc_macro2::{Span, TokenStream};

/// A convenience type for working with identifiers we write out in our macros.
#[derive(Debug, Clone)]
pub struct Ident(String, Option<Span>);

impl Ident {
    pub fn new(s: &str) -> Self {
        Ident(s.to_string(), None)
    }

    pub fn new_spanned(s: &str, span: Span) -> Ident {
        Ident(s.to_string(), Some(span))
    }

    pub fn for_inbuilt_scalar(s: &str) -> Self {
        Ident(transform_keywords(s.to_string()), None)
    }

    pub fn for_type(s: &str) -> Self {
        Ident(transform_keywords(s.to_pascal_case()), None)
    }

    pub fn for_field(s: &str) -> Self {
        Ident(transform_keywords(s.to_snake_case()), None)
    }

    pub fn for_module(s: &str) -> Self {
        let ident = s.to_snake_case();
        if ident == "super" {
            // This is an allowed keyword for modules.
            Ident(ident, None)
        } else {
            Ident(transform_keywords(ident), None)
        }
    }
}

impl PartialEq for Ident {
    fn eq(&self, other: &Ident) -> bool {
        // We only care about the ident itself for comparisons
        self.0 == other.0
    }
}

impl Eq for Ident {}

impl std::hash::Hash for Ident {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
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

impl From<proc_macro2::Ident> for Ident {
    fn from(ident: proc_macro2::Ident) -> Ident {
        Ident::new(&ident.to_string())
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
        use quote::{quote_spanned, TokenStreamExt};

        let macro_ident: proc_macro2::Ident = self.into();
        if let Some(span) = self.1 {
            tokens.append_all(quote_spanned! {span => #macro_ident })
        } else {
            macro_ident.to_tokens(tokens);
        }
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
