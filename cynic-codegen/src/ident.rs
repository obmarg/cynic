use inflector::Inflector;
use lazy_static::lazy_static;
use proc_macro2::{Span, TokenStream};
use std::collections::HashSet;

/// A convenience type for working with identifiers we write out in our macros.
#[derive(Debug, Clone)]
pub struct Ident(String, Option<Span>);

impl Ident {
    pub fn new<T: Into<String>>(s: T) -> Self {
        Ident(s.into(), None)
    }

    pub fn new_spanned<T: Into<String>>(s: T, span: Span) -> Ident {
        Ident(s.into(), Some(span))
    }

    pub fn for_inbuilt_scalar<T: Into<String>>(s: T) -> Self {
        Ident(transform_keywords(s.into()), None)
    }

    pub fn for_type<T: AsRef<str>>(s: T) -> Self {
        Ident(transform_keywords(s.as_ref().to_pascal_case()), None)
    }

    pub fn for_field<T: AsRef<str>>(s: T) -> Self {
        Ident(transform_keywords(s.as_ref().to_snake_case()), None)
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

lazy_static! {
    // A list of keywords in rust,
    // Taken from https://doc.rust-lang.org/reference/keywords.html
    static ref KEYWORDS: HashSet<&'static str> = {
        let mut set = HashSet::new();

        // Strict Keywords 2015
        set.insert("as");
        set.insert("break");
        set.insert("const");
        set.insert("continue");
        set.insert("crate");
        set.insert("else");
        set.insert("enum");
        set.insert("extern");
        set.insert("false");
        set.insert("fn");
        set.insert("for");
        set.insert("if");
        set.insert("impl");
        set.insert("in");
        set.insert("let");
        set.insert("loop");
        set.insert("match");
        set.insert("mod");
        set.insert("move");
        set.insert("mut");
        set.insert("pub");
        set.insert("ref");
        set.insert("return");
        set.insert("self");
        set.insert("Self");
        set.insert("static");
        set.insert("struct");
        set.insert("super");
        set.insert("trait");
        set.insert("true");
        set.insert("type");
        set.insert("unsafe");
        set.insert("use");
        set.insert("where");
        set.insert("while");

        // Strict keywords 2018
        set.insert("async");
        set.insert("await");
        set.insert("dyn");

        // Reserved Keywords 2015
        set.insert("abstract");
        set.insert("become");
        set.insert("box");
        set.insert("do");
        set.insert("final");
        set.insert("macro");
        set.insert("override");
        set.insert("priv");
        set.insert("typeof");
        set.insert("unsized");
        set.insert("virtual");
        set.insert("yield");

        // Reserved Keywords 2018
        set.insert("try");

        set
    };
}

fn transform_keywords(mut s: String) -> String {
    let s_ref: &str = &s;
    if KEYWORDS.contains(s_ref) {
        s.push('_');
    }

    s
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
