#![feature(proc_macro_span)]
#![feature(proc_macro_diagnostic)]
#![allow(clippy::let_and_return)]

extern crate proc_macro;

use std::{path::Path, rc::Rc};

use cargo_toml::Manifest;
use cynic_querygen::{indented, parse_query_document, Error, TypeIndex};
use graphql_parser::Pos;
use proc_macro::{Span, TokenStream};
use quote::quote;
use serde::Deserialize;

use cynic_codegen::{
    enum_derive, fragment_arguments_derive, fragment_derive, inline_fragments_derive,
    input_object_derive, scalar_derive, schema_for_derives, use_schema,
};

/// Imports a schema for use by cynic.
///
/// This creates all the required type markers & selection builder structures
/// required to use cynic with a given schema.  It should usually be called
/// in a module named schema, as the only statement in that module
///
/// ```rust,ignore
/// mod schema {
///     cynic::use_schema!("../schemas/starwars.schema.graphql");
/// }
/// ```
#[proc_macro]
pub fn use_schema(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as use_schema::QueryDslParams);

    let rv = use_schema::use_schema(input).unwrap().into();

    //eprintln!("{}", rv);

    rv
}

#[proc_macro_derive(QueryFragment, attributes(cynic, arguments))]
pub fn query_fragment_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let rv = match fragment_derive::fragment_derive(&ast) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    };

    //eprintln!("{}", rv);

    rv
}

#[proc_macro_derive(FragmentArguments)]
pub fn fragment_arguments_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let rv = match fragment_arguments_derive::fragment_arguments_derive(&ast) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    };

    rv
}

#[proc_macro_derive(InlineFragments, attributes(cynic))]
pub fn inline_fragments_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let rv = match inline_fragments_derive::inline_fragments_derive(&ast) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_errors().into(),
    };

    //eprintln!("{}", rv);

    rv
}

#[proc_macro_derive(Enum, attributes(cynic))]
pub fn enum_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let rv = match enum_derive::enum_derive(&ast) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    };

    //eprintln!("{}", rv);

    rv
}

#[proc_macro_derive(Scalar, attributes(cynic))]
pub fn scalar_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let rv = match scalar_derive::scalar_derive(&ast) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    };

    //eprintln!("{}", rv);

    rv
}

#[proc_macro_derive(InputObject, attributes(cynic))]
pub fn input_object_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let rv = match input_object_derive::input_object_derive(&ast) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    };

    //eprintln!("{}", rv);

    rv
}

#[proc_macro_attribute]
#[deprecated(
    since = "0.13.0",
    note = "query_module is deprecated, use schema_for_derives instead"
)]
pub fn query_module(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let module = syn::parse_macro_input!(input as syn::ItemMod);
    let attrs = syn::parse_macro_input!(attrs as syn::AttributeArgs);

    #[allow(deprecated)]
    let rv: TokenStream = match schema_for_derives::add_schema_attrs_via_query_module(attrs, module)
    {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    };

    // eprintln!("{}", rv);

    rv
}

#[proc_macro_attribute]
pub fn schema_for_derives(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let module = syn::parse_macro_input!(input as syn::ItemMod);
    let attrs = syn::parse_macro_input!(attrs as syn::AttributeArgs);

    let rv: TokenStream = match schema_for_derives::add_schema_attrs_to_derives(attrs, module) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    };

    // eprintln!("{}", rv);

    rv
}

#[proc_macro]
pub fn gql(input: TokenStream) -> TokenStream {
    // eprintln!("============================================================");
    let package_root = std::env::var("CARGO_MANIFEST_DIR")
        .expect("You are trying to call this from outside of cargo.");
    let path = extract_schema(&package_root)
        .expect("No schema path found in the Cargo.toml. Use `metadata.cynic.schema`.");
    let schema = std::fs::read_to_string(Path::new(&package_root).join(&path))
        .expect("Schema could not be read.");

    let input_clone = input.clone();

    let mut iter = input.into_iter();
    let first = iter.next().unwrap();
    let first_clone = first.clone();
    let last = iter.last().unwrap_or_else(|| first_clone);

    let source = first
        .span()
        .join(last.span())
        .map(|s| s.source_text())
        .flatten()
        .unwrap();

    let query_string = source.clone();
    eprintln!("{:?}", query_string);

    let fragments = document_to_fragment_structs(
        &query_string,
        &schema,
        Path::new(&package_root).join(path).to_string_lossy(),
    );

    let fragments = match fragments {
        Ok(fragments) => fragments,
        Err(error) => match error {
            Error::UnknownField(field, object, span) => {
                eprintln!("{:?}", span);
                let (span_start, span_end) =
                    get_span_from_pos(query_string, &first.span(), span, field.len());

                eprintln!("{:?}, {:?}", span_start, span_end);

                find_full_span_in_stream(input_clone, span_start, span_end).map(|e| {
                    e.error(format!(
                        "`{}` here was not found on the `{}` object.",
                        field, object
                    ))
                    .emit()
                });
                return quote! {}.into();
            }
            Error::ArgumentNotEnum(field, span) => {
                eprintln!("{:?}", span);
                let (span_start, span_end) =
                    get_span_from_pos(query_string, &first.span(), span, field.len());

                eprintln!("{:?}, {:?}", span_start, span_end);

                find_full_span_in_stream(input_clone, span_start, span_end).map(|e| {
                    e.error(format!(
                        "`{}` here was used like an enum but is no enum.",
                        field
                    ))
                    .emit()
                });
                return quote! {}.into();
            }
            e => {
                let error = e.to_string();
                return quote! { compile_error!(#error) }.into();
            }
        },
    };

    let stream: proc_macro::TokenStream = fragments.parse().unwrap();

    stream
}

fn get_span_from_pos(
    query_string: impl AsRef<str>,
    first: &Span,
    pos: Pos,
    len: usize,
) -> (usize, usize) {
    let span_start = query_string
        .as_ref()
        .lines()
        .take(pos.line - 1)
        .fold(0, |c, l| c + l.len()) // Add all the characters from the first N full lines.
        + pos.column // Add the characters from the started line.
        + (pos.line - 1) // Add all the newlines (one from each full line).
        + extract_span(&first).0; // Add offset to where the macro content token tree starts in the source code.
    let span_end = span_start + len - 1;
    (span_start, span_end)
}

/// Find the [`Span`] which contains the _Rust token_ at `pos` in the given [`TokenStream`].
fn find_span_in_stream(input: TokenStream, pos: usize) -> Option<Span> {
    for token in input {
        use proc_macro::TokenTree;
        match token {
            TokenTree::Group(ref t) => {
                if let Some(span) = find_span_in_stream(t.stream(), pos) {
                    return Some(span);
                }
            }
            _ => {}
        }

        let (start, end) = extract_span(&token.span());
        if start <= pos && pos <= end {
            return Some(token.span());
        }
    }
    None
}

/// Find the span of _Rust tokens_ which contains both `start` and `end` in the given [`TokenStream`].
fn find_full_span_in_stream(input: TokenStream, start: usize, end: usize) -> Option<Span> {
    find_span_in_stream(input.clone(), start)
        .and_then(|s1| find_span_in_stream(input, end).map(|s2| s1.join(s2)))
        .flatten()
}

/// Extracts the start and end position of the given [`Span`].
fn extract_span(span: &Span) -> (usize, usize) {
    let span = format!("{:?}", span);
    let split = span.split_at(9);
    let mut split = split.1.split("..");
    let start = split.next().unwrap().parse().unwrap();
    let second = split.next().unwrap();
    let end = second.split(")").next().unwrap().parse().unwrap();
    (start, end)
}

/// Reads the schema path from `Cargo.toml/metadata.cynic.schema`.
fn extract_schema(package_root: impl AsRef<Path>) -> Option<String> {
    let cargo_toml = package_root.as_ref().join("Cargo.toml");
    let cargo_toml_content = std::fs::read(&cargo_toml).expect("Cargo.toml could not be read.");

    let meta = Manifest::<Metadata>::from_slice_with_metadata(&cargo_toml_content)
        .expect("Cargo.toml could not be parsed.");
    let schema = meta
        .package
        .map(|p| {
            p.metadata
                .map(|m| m.cynic.map(|m| m.schema).flatten())
                .flatten()
        })
        .flatten();

    return schema;
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct Metadata {
    pub cynic: Option<CynicMetadata>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct CynicMetadata {
    pub schema: Option<String>,
}

fn document_to_fragment_structs(
    query: impl AsRef<str>,
    schema: impl AsRef<str>,
    schema_path: impl AsRef<str>,
) -> Result<String, Error> {
    use std::fmt::Write;

    let schema = graphql_parser::parse_schema::<&str>(schema.as_ref())?;
    let query = graphql_parser::parse_query::<&str>(query.as_ref())?;

    eprintln!("{:#?}", query);

    let type_index = Rc::new(TypeIndex::from_schema(&schema));
    let parsed_output = parse_query_document(&query, &type_index)?;

    let mut output = String::new();

    let operation = parsed_output
        .normalised_document
        .operations
        .first()
        .unwrap();
    let name = operation.name.unwrap();

    writeln!(output, "#[cynic::schema_for_derives(").unwrap();
    writeln!(output, "    file = r#\"{}\"#,", schema_path.as_ref()).unwrap();
    writeln!(output, "    module = \"gql_schema\",").unwrap();
    writeln!(output, ")]\nmod {} {{", name).unwrap();

    let mod_output = &mut indented(&mut output, 4);

    writeln!(mod_output, "use crate::gql_schema;\n").unwrap();

    for argument_struct in &parsed_output.argument_structs {
        writeln!(mod_output, "{}", argument_struct).unwrap();
    }

    for fragment in parsed_output.query_fragments {
        fragment.fmt(mod_output)?;
    }

    for en in parsed_output.enums {
        writeln!(mod_output, "{}", en).unwrap();
    }

    for input_object in parsed_output.input_objects {
        writeln!(mod_output, "{}", input_object).unwrap();
    }

    for scalar in parsed_output.scalars {
        writeln!(mod_output, "{}", scalar).unwrap();
    }

    write!(mod_output, "pub fn query(").unwrap();

    let arguments = if let Some(argument_struct) = parsed_output.argument_structs.iter().next() {
        let mut output = String::new();

        write!(output, "{} {{", argument_struct.name).unwrap();

        for field in &argument_struct.fields {
            write!(mod_output, "{}: {},", field.name(), field.type_spec()).unwrap();
            write!(output, "{},", field.name()).unwrap();
        }

        write!(output, "}}").unwrap();

        output
    } else {
        "()".into()
    };

    writeln!(mod_output, ") -> cynic::GraphQlResponse<{}> {{", name).unwrap();
    writeln!(
        mod_output,
        r#"    use cynic::QueryBuilder;

    let query = {}::build(&{});
    crate::gql_schema::run_query(query)"#,
        name, arguments
    )
    .unwrap();
    writeln!(mod_output, "}}").unwrap();

    writeln!(output, "}}\n").unwrap();

    // eprintln!("{}", output);

    Ok(output)
}
