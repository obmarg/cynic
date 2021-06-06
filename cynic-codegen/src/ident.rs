use inflector::Inflector;
use lazy_static::lazy_static;
use proc_macro2::{Span, TokenStream};
use quote::format_ident;
use std::{borrow::Cow, collections::HashSet};

/// A convenience type for working with identifiers we write out in our macros.
#[derive(Debug, Clone)]
pub struct Ident {
    rust: proc_macro2::Ident,
    graphql: String,
    span: Option<Span>,
}

impl Ident {
    pub fn new<T: Into<String>>(s: T) -> Self {
        let s = s.into();

        Ident {
            rust: format_ident!("{}", transform_keywords(&s)),
            graphql: s,
            span: None,
        }
    }

    pub fn new_spanned<T: Into<String>>(s: T, span: Span) -> Ident {
        Ident {
            span: Some(span),
            ..Ident::new(s)
        }
    }

    pub fn from_proc_macro2(
        ident: &proc_macro2::Ident,
        rename: impl Into<Option<RenameRule>>,
    ) -> Self {
        let ident_str = ident.to_string();
        let graphql_name = if ident_str.starts_with("r#") {
            // This is a raw identifier so strip the r# off...
            ident_str.strip_prefix("r#").unwrap()
        } else {
            &ident_str
        };

        Ident {
            rust: ident.clone(),
            graphql: rename
                .into()
                .map(|r| r.apply(graphql_name))
                .unwrap_or_else(|| graphql_name.to_string()),
            span: Some(ident.span()),
        }
    }

    pub fn for_inbuilt_scalar<T: Into<String>>(s: T) -> Self {
        Ident::new(s)
    }

    pub fn for_type<T: AsRef<str>>(s: T) -> Self {
        Ident::new(s.as_ref().to_pascal_case())
    }

    pub fn for_variant(s: impl AsRef<str>) -> Self {
        Ident::new(s.as_ref().to_pascal_case())
    }

    pub fn for_field<T: AsRef<str>>(s: T) -> Self {
        let s = s.as_ref();
        if s == "_" {
            Ident {
                rust: format_ident!("__underscore"),
                graphql: "_".to_string(),
                span: None,
            }
        } else {
            Ident::new(to_snake_case(s))
        }
    }

    pub fn for_module(s: &str) -> Self {
        Ident::new(s.to_snake_case())
    }

    pub fn rust_name(&self) -> String {
        self.rust.to_string()
    }

    pub fn graphql_name(&self) -> &str {
        &self.graphql
    }

    pub fn with_span(self, span: Span) -> Self {
        Self {
            span: Some(span),
            ..self
        }
    }

    pub fn span(&self) -> Span {
        self.span.clone().unwrap_or_else(Span::call_site)
    }
}

impl PartialEq for Ident {
    fn eq(&self, other: &Ident) -> bool {
        // We only care about the GraphQL ident for comparison purposes.
        self.graphql == other.graphql
    }
}

impl Eq for Ident {}

impl std::hash::Hash for Ident {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // We only care about the GraphQL ident for hashing purposes.
        self.graphql.hash(state);
    }
}

impl From<proc_macro2::Ident> for Ident {
    fn from(ident: proc_macro2::Ident) -> Ident {
        Ident::from_proc_macro2(&ident, None)
    }
}

impl From<&Ident> for proc_macro2::Ident {
    fn from(ident: &Ident) -> proc_macro2::Ident {
        ident.rust.clone()
    }
}

impl quote::ToTokens for Ident {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote_spanned, TokenStreamExt};

        let macro_ident: proc_macro2::Ident = self.into();
        if let Some(span) = self.span {
            tokens.append_all(quote_spanned! {span => #macro_ident })
        } else {
            macro_ident.to_tokens(tokens);
        }
    }
}

#[derive(Debug)]
pub enum RenameRule {
    RenameAll(RenameAll),
    RenameTo(String),
}

impl RenameRule {
    pub fn new(all: RenameAll, specific: Option<impl AsRef<String>>) -> Option<RenameRule> {
        match (specific, all) {
            (Some(specific), _) => Some(RenameRule::RenameTo(specific.as_ref().to_string())),
            (_, RenameAll::None) => None,
            (_, all) => Some(RenameRule::RenameAll(all)),
        }
    }

    fn apply(&self, string: impl AsRef<str>) -> String {
        match self {
            RenameRule::RenameTo(s) => s.clone(),
            RenameRule::RenameAll(RenameAll::Lowercase) => string.as_ref().to_lowercase(),
            RenameRule::RenameAll(RenameAll::Uppercase) => string.as_ref().to_uppercase(),
            RenameRule::RenameAll(RenameAll::PascalCase) => string.as_ref().to_pascal_case(),
            RenameRule::RenameAll(RenameAll::CamelCase) => string.as_ref().to_camel_case(),
            RenameRule::RenameAll(RenameAll::SnakeCase) => string.as_ref().to_snake_case(),
            RenameRule::RenameAll(RenameAll::ScreamingSnakeCase) => {
                string.as_ref().to_screaming_snake_case()
            }
            RenameRule::RenameAll(RenameAll::None) => {
                panic!("RenameRule::new not filtering out RenameAll::None properly!")
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// Rules to rename all fields in an InputObject or variants in an Enum
/// as GraphQL naming conventions usually don't match rust
pub enum RenameAll {
    None,
    /// For names that are entirely lowercase in GraphQL: `myfield`
    Lowercase,
    /// For names that are entirely uppercase in GraphQL: `MYFIELD`
    Uppercase,
    /// For names that are entirely pascal case in GraphQL: `MyField`
    PascalCase,
    /// For names that are entirely camel case in GraphQL: `myField`
    CamelCase,
    /// For names that are entirely snake case in GraphQL: `my_field`
    SnakeCase,
    /// For names that are entirely snake case in GraphQL: `MY_FIELD`
    ScreamingSnakeCase,
}

impl darling::FromMeta for RenameAll {
    fn from_string(value: &str) -> Result<RenameAll, darling::Error> {
        match value.to_lowercase().as_ref() {
            "none" => Ok(RenameAll::None),
            "lowercase" => Ok(RenameAll::Lowercase),
            "uppercase" => Ok(RenameAll::Uppercase),
            "pascalcase" => Ok(RenameAll::PascalCase),
            "camelcase" => Ok(RenameAll::CamelCase),
            "snake_case" => Ok(RenameAll::SnakeCase),
            "screaming_snake_case" => Ok(RenameAll::ScreamingSnakeCase),
            _ => {
                // Feels like it'd be nice if this error listed all the options...
                Err(darling::Error::unknown_value(value))
            }
        }
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

fn transform_keywords(s: &str) -> Cow<str> {
    let s_ref: &str = &s;
    if KEYWORDS.contains(s_ref) {
        format!("r#{}", s).into()
    } else {
        s.into()
    }
}

fn to_snake_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    // Setting this to true to avoid adding underscores at the beginning
    let mut prev_is_upper = true;
    for c in s.chars() {
        if c.is_uppercase() && !prev_is_upper {
            buf.push('_');
            buf.extend(c.to_lowercase());
            prev_is_upper = true;
        } else if c.is_uppercase() {
            buf.extend(c.to_lowercase());
        } else {
            prev_is_upper = false;
            buf.push(c);
        }
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_proc_macro_2() {
        let ident = Ident::from_proc_macro2(&format_ident!("r#test"), None);
        assert_eq!(ident.graphql, "test");
        assert_eq!(ident.rust, format_ident!("r#test"));

        let ident = Ident::from_proc_macro2(&format_ident!("test"), None);
        assert_eq!(ident.graphql, "test");
        assert_eq!(ident.rust, format_ident!("test"));
    }

    #[test]
    fn test_new() {
        let ident = Ident::new("test");
        assert_eq!(ident.graphql, "test");
        assert_eq!(ident.rust, format_ident!("test"));

        let ident = Ident::new("type");
        assert_eq!(ident.graphql, "type");
        assert_eq!(ident.rust, format_ident!("r#type"));
    }

    #[test]
    fn test_transform_keywords() {
        assert_eq!(transform_keywords("test"), "test");
        assert_eq!(transform_keywords("type"), "r#type");
    }

    #[test]
    fn test_underscore() {
        assert_eq!(to_snake_case("_hello"), "_hello");
        assert_eq!(to_snake_case("_"), "_");
        assert_eq!(Ident::for_field("_"), Ident::new("_"));
    }
}
