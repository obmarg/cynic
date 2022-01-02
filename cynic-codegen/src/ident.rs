use lazy_static::lazy_static;
use proc_macro2::{Span, TokenStream};
use quote::format_ident;
use std::{
    borrow::{Borrow, Cow},
    collections::HashSet,
};

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
        let s = s.as_ref();
        if s == "_" {
            Ident {
                rust: format_ident!("Underscore"),
                graphql: "_".to_string(),
                span: None,
            }
        } else {
            Ident::new(to_pascal_case(s))
        }
    }

    pub fn for_variant(s: impl AsRef<str>) -> Self {
        Ident::new(to_pascal_case(s.as_ref()))
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
        Ident::new(to_snake_case(s))
    }

    pub fn rust_name(&self) -> String {
        self.rust.to_string()
    }

    pub fn as_field_module(&self) -> Ident {
        Ident {
            rust: format_ident!("{}_fields", to_snake_case(&self.rust_name())),
            graphql: self.graphql.clone(),
            span: self.span,
        }
    }

    pub fn as_field_marker_type(&self) -> Ident {
        Ident {
            rust: format_ident!("{}", to_pascal_case(&self.rust_name())),
            graphql: self.graphql.clone(),
            span: self.span,
        }
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
        self.span.unwrap_or_else(Span::call_site)
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

impl std::cmp::PartialOrd for Ident {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // We only care about the GraphQL ident for ordering purposes.
        self.graphql.partial_cmp(&other.graphql)
    }
}

impl std::cmp::Ord for Ident {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // We only care about the GraphQL ident for ordering purposes.
        self.graphql.cmp(&other.graphql)
    }
}

impl AsRef<Ident> for Ident {
    fn as_ref(&self) -> &Ident {
        self
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

impl From<Ident> for proc_macro2::Ident {
    fn from(ident: Ident) -> proc_macro2::Ident {
        ident.rust
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
            RenameRule::RenameAll(RenameAll::PascalCase) => to_pascal_case(string.as_ref()),
            RenameRule::RenameAll(RenameAll::CamelCase) => to_camel_case(string.as_ref()),
            RenameRule::RenameAll(RenameAll::SnakeCase) => to_snake_case(string.as_ref()),
            RenameRule::RenameAll(RenameAll::ScreamingSnakeCase) => {
                to_snake_case(string.as_ref()).to_uppercase()
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
    // A list of keywords in rust that can be converted to raw identifiers
    // Taken from https://doc.rust-lang.org/reference/keywords.html
    static ref RAW_KEYWORDS: HashSet<&'static str> = {
        let mut set = HashSet::new();

        // Strict Keywords 2015
        set.insert("as");
        set.insert("break");
        set.insert("const");
        set.insert("continue");
        set.insert("else");
        set.insert("enum");
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
        set.insert("static");
        set.insert("struct");
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

lazy_static! {
    // A list of keywords in rust that cannot be converted to raw identifiers
    // Taken from https://github.com/rust-lang/rust/blob/1.31.1/src/libsyntax_pos/symbol.rs#L456-L460
    static ref NON_RAW_KEYWORDS: HashSet<&'static str> = {
        let mut set = HashSet::new();

        set.insert("super");
        set.insert("self");
        set.insert("Self");
        set.insert("extern");
        set.insert("crate");

        set
    };
}

fn transform_keywords(s: &str) -> Cow<str> {
    let s_ref: &str = s;
    if NON_RAW_KEYWORDS.contains(s_ref) {
        format!("{}_", s).into()
    } else if RAW_KEYWORDS.contains(s_ref) {
        format!("r#{}", s).into()
    } else {
        s.into()
    }
}

pub fn to_snake_case(s: &str) -> String {
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

fn to_pascal_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut first_char = true;
    let mut prev_is_upper = false;
    let mut prev_is_underscore = false;
    for c in s.chars() {
        if first_char {
            if c == '_' {
                prev_is_underscore = true;
            } else if c.is_uppercase() {
                prev_is_upper = true;
                buf.push(c);
            } else {
                buf.extend(c.to_uppercase());
            }
            first_char = false;
            continue;
        }

        if c.is_uppercase() {
            if prev_is_upper {
                buf.extend(c.to_lowercase());
            } else {
                buf.push(c);
            }
            prev_is_upper = true;
        } else if c == '_' {
            prev_is_underscore = true;
        } else {
            if prev_is_upper {
                buf.extend(c.to_lowercase())
            } else if prev_is_underscore {
                buf.extend(c.to_uppercase());
            } else {
                buf.push(c);
            }
            prev_is_upper = false;
            prev_is_underscore = false;
        }
    }

    buf
}

fn to_camel_case(s: &str) -> String {
    let s = to_pascal_case(s);

    let mut buf = String::with_capacity(s.len());
    let mut chars = s.chars();

    if let Some(first_char) = chars.next() {
        buf.extend(first_char.to_lowercase());
    }

    buf.extend(chars);

    buf
}

pub trait PathExt {
    fn push(&mut self, ident: impl Borrow<super::Ident>);
}

impl PathExt for syn::Path {
    fn push(&mut self, ident: impl Borrow<crate::Ident>) {
        self.segments.push(ident.borrow().rust.clone().into())
    }
}

pub fn empty_path() -> syn::Path {
    syn::Path {
        leading_colon: None,
        segments: syn::punctuated::Punctuated::new(),
    }
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

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("aString"), "a_string");
        assert_eq!(to_snake_case("MyString"), "my_string");
        assert_eq!(to_snake_case("my_string"), "my_string");
        assert_eq!(to_snake_case("_another_one"), "_another_one");
        assert_eq!(to_snake_case("RepeatedUPPERCASE"), "repeated_uppercase");
        assert_eq!(to_snake_case("UUID"), "uuid");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("aString"), "aString");
        assert_eq!(to_camel_case("MyString"), "myString");
        assert_eq!(to_camel_case("my_string"), "myString");
        assert_eq!(to_camel_case("_another_one"), "anotherOne");
        assert_eq!(to_camel_case("RepeatedUPPERCASE"), "repeatedUppercase");
        assert_eq!(to_camel_case("UUID"), "uuid");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("aString"), "AString");
        assert_eq!(to_pascal_case("MyString"), "MyString");
        assert_eq!(to_pascal_case("my_string"), "MyString");
        assert_eq!(to_pascal_case("_another_one"), "AnotherOne");
        assert_eq!(to_pascal_case("RepeatedUPPERCASE"), "RepeatedUppercase");
        assert_eq!(to_pascal_case("UUID"), "Uuid");
    }

    #[test]
    fn casings_are_not_lossy_where_possible() {
        for s in ["snake_case_thing", "snake"] {
            assert_eq!(to_snake_case(&to_pascal_case(s)), s);
        }

        for s in ["PascalCase", "Pascal"] {
            assert_eq!(to_pascal_case(&to_snake_case(s)), s);
        }

        for s in ["camelCase", "camel"] {
            assert_eq!(to_camel_case(&to_snake_case(s)), s);
        }
    }
}
