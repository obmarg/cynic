use std::{
    fs::{self, File},
    io::Read,
};

use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use ungrammar::{Grammar, Rule, TokenData};

impl crate::flags::Codegen {
    pub fn run(self) -> Result<()> {
        let mut grammar_data = String::new();
        File::open("cynic-parser/src/query.ungram")?.read_to_string(&mut grammar_data)?;

        let grammar = grammar_data.parse::<Grammar>()?;

        gen_tokens(&grammar)?;
        gen_nodes(&grammar)?;

        Ok(())
    }
}

fn gen_nodes(grammar: &Grammar) -> Result<()> {
    let ast = grammar_to_ast(grammar);

    let enums = ast.enums.into_iter().map(gen_enum_node);
    let nodes = ast.nodes.into_iter().map(gen_node);

    let pretty = format_code(
        &quote! {
            use crate::{
                syntax::{SyntaxKind::{self, *}, SyntaxToken, SyntaxNode},
                ast::{AstNode, AstChildren, NameOwner, support},
            };

            #(#nodes)*
            #(#enums)*
        }
        .to_string(),
    )?
    .replace("#[derive", "\n#[derive");

    fs::write("cynic-parser/src/ast/generated/nodes.rs", pretty)?;

    Ok(())
}

fn gen_enum_node(en: AstEnumSrc) -> TokenStream {
    let variants = en
        .variants
        .iter()
        .map(|v| format_ident!("{}", v))
        .collect::<Vec<_>>();
    let name = format_ident!("{}", en.name);
    let kinds = variants
        .iter()
        .map(|name| format_ident!("{}", to_upper_snake_case(&name.to_string())))
        .collect::<Vec<_>>();

    let variants_lower = en
        .variants
        .iter()
        .map(|v| format_ident!("{}", to_lower_snake_case(v)))
        .collect::<Vec<_>>();

    let is_variants = en
        .variants
        .iter()
        .map(|v| format_ident!("is_{}", to_lower_snake_case(v)))
        .collect::<Vec<_>>();

    quote! {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum #name {
            #(#variants(#variants),)*
        }
        impl #name {
            #(
                pub fn #variants_lower(&self) -> Option<#variants> {
                    match self {
                        Self::#variants(inner) => Some(inner.clone()),
                        _ => None
                    }
                }

                pub fn #is_variants(&self) -> bool {
                    matches!(self, Self::#variants(_))
                }
            )*
        }

        impl AstNode for #name {
            fn can_cast(kind: SyntaxKind) -> bool {
                matches!(kind, #(#kinds)|*)
            }
            fn cast(syntax: SyntaxNode) -> Option<Self> {
                let res = match syntax.kind() {
                    #(
                    #kinds => #name::#variants(#variants { syntax }),
                    )*
                    _ => return None,
                };
                Some(res)
            }
            fn syntax(&self) -> &SyntaxNode {
                match self {
                    #(
                    #name::#variants(it) => &it.syntax,
                    )*
                }
            }
        }

    }
}

fn gen_node(node: AstNodeSrc) -> TokenStream {
    let name = format_ident!("{}", node.name);
    let kind = format_ident!("{}", to_upper_snake_case(&node.name));

    let methods = node.fields.iter().map(|field| {
        let method_name = field.method_name();
        let ty = field.ty();
        if field.is_many() {
            quote! {
                pub fn #method_name(&self) -> AstChildren<#ty> {
                    support::children(&self.syntax)
                }
            }
        } else if let Some(token_kind) = field.token_kind() {
            quote! {
                pub fn #method_name(&self) -> Option<#ty> {
                    support::token(&self.syntax, #token_kind)
                }
            }
        } else {
            quote! {
                pub fn #method_name(&self) -> Option<#ty> {
                    support::child(&self.syntax)
                }
            }
        }
    });

    let impls = node.impls.iter().map(|imp| {
        let ident = format_ident!("{}", imp);
        quote! { impl #ident for #name {} }
    });

    quote! {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct #name {
            pub(crate) syntax: SyntaxNode
        }

        impl #name {
            #(#methods)*
        }

        impl AstNode for #name {
            fn can_cast(kind: SyntaxKind) -> bool {
                kind == #kind
            }
            fn cast(syntax: SyntaxNode) -> Option<Self> {
                if Self::can_cast(syntax.kind()) { Some(Self { syntax }) } else { None }
            }
            fn syntax(&self) -> &SyntaxNode { &self.syntax }
        }

        #(#impls)*
    }
}

#[derive(Default)]
struct AstSrc {
    nodes: Vec<AstNodeSrc>,
    enums: Vec<AstEnumSrc>,
}

struct AstNodeSrc {
    name: String,
    fields: Vec<AstFieldSrc>,
    impls: Vec<&'static str>,
}

enum AstFieldSrc {
    Node {
        name: String,
        ty: String,
        cardinality: Cardinality,
    },
    Token(String),
}

impl AstFieldSrc {
    fn name_or_tok(&self) -> &str {
        match self {
            AstFieldSrc::Node { name, .. } => name,
            AstFieldSrc::Token(tok) => tok,
        }
    }

    fn is_same(&self, other: &AstFieldSrc) -> bool {
        match (self, other) {
            (AstFieldSrc::Node { .. }, AstFieldSrc::Node { .. }) => {
                self.name_or_tok() == other.name_or_tok()
            }
            (AstFieldSrc::Token(_), AstFieldSrc::Token(_)) => {
                self.name_or_tok() == other.name_or_tok()
            }
            _ => false,
        }
    }

    fn is_many(&self) -> bool {
        matches!(
            self,
            AstFieldSrc::Node {
                cardinality: Cardinality::Many,
                ..
            }
        )
    }

    fn token_kind(&self) -> Option<proc_macro2::TokenStream> {
        match self {
            AstFieldSrc::Token(token) => {
                let name = format_ident!("{}", to_upper_snake_case(&token_name(token)));
                Some(quote! { #name })
            }
            _ => None,
        }
    }

    fn method_name(&self) -> proc_macro2::Ident {
        match self {
            AstFieldSrc::Token(name) => {
                let name = to_lower_snake_case(&token_name(&name));
                format_ident!("{}_token", name)
            }
            AstFieldSrc::Node { name, .. } => {
                if name == "type" {
                    format_ident!("ty")
                } else {
                    format_ident!("{}", name)
                }
            }
        }
    }

    fn ty(&self) -> proc_macro2::Ident {
        match self {
            AstFieldSrc::Token(_) => format_ident!("SyntaxToken"),
            AstFieldSrc::Node { ty, .. } => format_ident!("{}", ty),
        }
    }
}

enum Cardinality {
    Optional,
    Many,
}

struct AstEnumSrc {
    name: String,
    variants: Vec<String>,
}

fn grammar_to_ast(grammar: &Grammar) -> AstSrc {
    let mut res = AstSrc::default();
    let nodes = grammar.iter().collect::<Vec<_>>();

    for node in nodes {
        let name = grammar[node].name.clone();
        let rule = &grammar[node].rule;
        match extract_enum_variants(grammar, rule) {
            Some(variants) => res.enums.push(AstEnumSrc { name, variants }),
            None => {
                let mut fields = Vec::new();
                extract_fields(&mut fields, grammar, rule, None);
                fields.sort_by_key(|field| field.name_or_tok().to_string());
                fields.dedup_by(|lhs, rhs| lhs.is_same(rhs));
                res.nodes.push(AstNodeSrc {
                    name,
                    fields,
                    impls: vec![],
                });
            }
        }
    }

    for node in &mut res.nodes {
        let name_token = node
            .fields
            .iter()
            .enumerate()
            .find(|(_, f)| matches!(*f, AstFieldSrc::Token(tok) if tok == "name"));

        if let Some((i, _)) = name_token {
            node.fields.remove(i);
            node.impls.push("NameOwner");
        }
    }

    res
}

fn extract_enum_variants(grammar: &Grammar, rule: &Rule) -> Option<Vec<String>> {
    let alternatives = match rule {
        Rule::Alt(it) => it,
        _ => return None,
    };
    let mut variants = Vec::new();
    for alt in alternatives {
        match alt {
            Rule::Node(it) => variants.push(grammar[*it].name.clone()),
            _ => return None,
        }
    }
    Some(variants)
}

fn extract_fields(
    fields: &mut Vec<AstFieldSrc>,
    grammar: &Grammar,
    rule: &Rule,
    label: Option<&str>,
) {
    match rule {
        Rule::Node(node) => {
            let ty = grammar[*node].name.clone();
            let name = to_lower_snake_case(&ty);
            fields.push(AstFieldSrc::Node {
                name,
                ty,
                cardinality: Cardinality::Optional,
            });
        }
        Rule::Token(token) => {
            fields.push(AstFieldSrc::Token(grammar[*token].name.clone()));
        }
        Rule::Rep(inner) => match &**inner {
            Rule::Node(node) => {
                let ty = grammar[*node].name.clone();
                fields.push(AstFieldSrc::Node {
                    name: label
                        .map(|l| l.to_string())
                        .unwrap_or_else(|| to_lower_snake_case(&ty)),
                    ty,
                    cardinality: Cardinality::Many,
                });
            }
            Rule::Token(tok) if grammar[*tok].name == "string_part" => {}
            _ => {
                panic!("Unimplemented rule: {:?}", rule)
            }
        },
        Rule::Seq(rules) | Rule::Alt(rules) => {
            for rule in rules {
                extract_fields(fields, grammar, rule, None)
            }
        }
        Rule::Opt(rule) => extract_fields(fields, grammar, rule, None),
        Rule::Labeled { label, rule } => {
            extract_fields(fields, grammar, rule, Some(label.as_ref()))
        }
    }
}

fn gen_tokens(grammar: &Grammar) -> Result<()> {
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

fn gen_token(token: &TokenData) -> TokenStream {
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

fn to_lower_snake_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev = false;
    for c in s.chars() {
        if c.is_ascii_uppercase() && prev {
            buf.push('_')
        }
        prev = true;

        buf.push(c.to_ascii_lowercase());
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
