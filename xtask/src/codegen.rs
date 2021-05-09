use std::{
    fs::{self, File},
    io::{Read, Write},
};

use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use ungrammar::{Grammar, TokenData};

impl crate::flags::Codegen {
    pub fn run(self) -> Result<()> {
        let mut grammar_data = String::new();
        File::open("cynic-parser/src/query.ungram")?.read_to_string(&mut grammar_data)?;

        let grammar = grammar_data.parse::<Grammar>()?;

        gen_tokens(&grammar)?;

        Ok(())
    }
}

pub fn gen_tokens(grammar: &Grammar) -> Result<()> {
    let tokens = grammar.tokens().map(|tok| gen_token(&grammar[tok]));

    let pretty = format_code(
        &quote! {
            use crate::{
                syntax::{SyntaxKind::{self, *}, SyntaxToken},
                ast::AstToken
            };

            #(#tokens)*
        }
        .to_string(),
    )?
    .replace("#[derive", "\n#[derive");

    fs::write("cynic-parser/src/ast/generated/tokens.rs", pretty)?;

    Ok(())
}

pub fn gen_token(token: &TokenData) -> TokenStream {
    let token_name = token_name(&token.name);
    let name = format_ident!("{}", token_name);
    let kind = format_ident!("{}", to_upper_snake_case(&token_name));

    quote! {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct #name {
            pub(crate) syntax: SyntaxToken,
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.syntax, f)
            }
        }

        impl AstToken for #name {
            fn can_cast(kind: SyntaxKind) -> bool { kind == #kind }
            fn cast(syntax: SyntaxToken) -> Option<Self> {
                if Self::can_cast(syntax.kind()) { Some(Self { syntax }) } else { None }
            }
            fn syntax(&self) -> &SyntaxToken { &self.syntax }
        }
    }
}

fn token_name(token: &str) -> String {
    match token {
        ":" => "Colon".into(),
        "$" => "Dollar".into(),
        "!" => "Bang".into(),
        "[" => "OpenSquare".into(),
        "]" => "CloseSquare".into(),
        "(" => "OpenParen".into(),
        ")" => "CloseParen".into(),
        "{" => "OpenCurly".into(),
        "}" => "CloseCurly".into(),
        "=" => "Equals".into(),
        "@" => "At".into(),
        "," => "Comma".into(),
        "-" => "NegativeSign".into(),
        "\"" => "Quote".into(),
        "\"\"\"" => "BlockQuote".into(),
        "true" => "True".into(),
        "false" => "False".into(),
        "null" => "Null".into(),
        "..." => "Spread".into(),
        other => to_pascal_case(other),
    }
}

fn to_upper_snake_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev = false;
    for c in s.chars() {
        if c.is_ascii_uppercase() && prev {
            buf.push('_')
        }
        prev = true;

        buf.push(c.to_ascii_uppercase());
    }
    buf
}

fn to_pascal_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev_is_underscore = true;
    for c in s.chars() {
        if c == '_' {
            prev_is_underscore = true;
        } else if prev_is_underscore {
            buf.push(c.to_ascii_uppercase());
            prev_is_underscore = false;
        } else {
            buf.push(c.to_ascii_lowercase());
        }
    }
    buf
}

fn format_code(text: &str) -> Result<String> {
    let stdout = xshell::cmd!("rustfmt").stdin(text).read()?;

    Ok(stdout)
}
