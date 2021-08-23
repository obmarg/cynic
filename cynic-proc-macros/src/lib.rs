#![feature(proc_macro_span)]
#![feature(proc_macro_diagnostic)]
#![allow(clippy::let_and_return)]

extern crate proc_macro;

use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::{path::Path, rc::Rc};

use cargo_toml::Manifest;
use cynic_querygen::{parse_query_document, Error, TypeIndex};
use graphql_parser::query::{Definition, OperationDefinition};
use graphql_parser::{error::Error as ConsumeError, Pos};
use proc_macro::{Span, TokenStream};
use proc_macro2::Ident;
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

    let mut iter = input.clone().into_iter();
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
    // eprintln!("{:?}", query_string);

    // let query_string = input.to_string();

    let fragments = document_to_fragment_structs(
        &query_string,
        &schema,
        Path::new(&package_root).join(path).to_string_lossy(),
    );

    let fragments = match fragments {
        Ok(fragments) => fragments,
        Err(error) => match error {
            Error::ParseError(error) => {
                let errors = &error.errors;
                let mut error_message = String::new();
                let mut iter = errors.iter().peekable();
                while let Some(error) = iter.next() {
                    use std::fmt::Write;
                    match error {
                        ConsumeError::Unexpected(token) => {
                            write!(error_message, "Unexpected `{}`. Expected ", token).unwrap();
                        }
                        ConsumeError::Expected(token) => {
                            write!(error_message, "`{}`", token).unwrap();
                            if let Some(_peek) = iter.peek() {
                                write!(error_message, ", ").unwrap();
                            } else {
                                write!(error_message, ".").unwrap();
                            }
                        }
                        ConsumeError::Message(_token) => todo!(),
                        ConsumeError::Other(_other) => todo!(),
                    }
                }

                let span = error.position;
                eprintln!("{:?}", span);
                let (span_start, span_end) =
                    get_span_from_pos(&query_string, &first.span(), span, 1);

                eprintln!("{:?}, {:?}", span_start, span_end);

                find_full_span_in_stream(input_clone.clone(), span_start, span_end)
                    .map(|e| e.error(error_message).emit());

                return quote! {}.into();
            }
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
                return quote! { compile_error!{ #error } }.into();
            }
        },
    };

    // eprintln!("{}", &fragments.to_string());

    let mut rustfmt = Command::new("rustfmt");
    let rustfmt = rustfmt
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("--edition")
        .arg("2018")
        .arg("--emit")
        .arg("stdout");

    let rustfmt = rustfmt.spawn().expect("Failed to run rustfmt");

    rustfmt
        .stdin
        .unwrap()
        .write_all(fragments.to_string().as_bytes())
        .unwrap();

    let mut fragment_string = String::new();
    rustfmt
        .stdout
        .unwrap()
        .read_to_string(&mut fragment_string)
        .unwrap();

    // eprintln!("{}", &fragment_string);

    fragments
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
) -> Result<TokenStream, Error> {
    let schema = graphql_parser::parse_schema::<&str>(schema.as_ref())?;
    let query = graphql_parser::parse_query::<&str>(query.as_ref())?;

    // eprintln!("{:#?}", query);

    let type_index = Rc::new(TypeIndex::from_schema(&schema));
    let parsed_output = parse_query_document(&query, &type_index)?;

    let operation = parsed_output
        .normalised_document
        .operations
        .first()
        .unwrap();
    let name = Ident::new(operation.name.unwrap(), proc_macro2::Span::call_site());

    let argument_structs = &parsed_output.argument_structs;
    let query_fragments = &parsed_output.query_fragments;
    let enums = &parsed_output.enums;
    let input_objects = &parsed_output.input_objects;

    let mut includes = vec![];
    for argument_struct in argument_structs {
        for field in &argument_struct.fields {
            for scalar in &parsed_output.scalars {
                if scalar.0 == field.type_spec() {
                    use inflector::Inflector;
                    includes.push({
                        let name =
                            Ident::new(&scalar.0.to_pascal_case(), proc_macro2::Span::call_site());
                        quote! {
                            use super::#name;
                        }
                    });
                }
            }
        }
    }

    let (struct_definition, arguments) =
        if let Some(argument_struct) = parsed_output.argument_structs.iter().next() {
            let struct_name = Ident::new(&argument_struct.name, proc_macro2::Span::call_site());

            let mut field_names = vec![];
            let mut field_types = vec![];

            for field in &argument_struct.fields {
                field_names.push(Ident::new(&field.name(), proc_macro2::Span::call_site()));
                field_types.push(
                    field
                        .type_spec()
                        .parse::<proc_macro2::TokenStream>()
                        .unwrap(),
                );
            }

            (
                Some(quote! { #struct_name {#(#field_names),*}}),
                Some(quote! { #(#field_names: #field_types),*}),
            )
        } else {
            (Some(quote! {()}), None)
        };

    let schema_path = schema_path.as_ref();

    let output_type = query_fragments
        .iter()
        .next()
        .and_then(|fragment| {
            fragment
                .fields
                .iter()
                .map(|field| field.field_type.type_spec())
                .next()
        })
        .unwrap_or_else(|| "()".to_string());
    let output_type = output_type.parse::<proc_macro2::TokenStream>().unwrap();

    let function = match &query.definitions[0] {
        Definition::Operation(OperationDefinition::Query(_)) => {
            quote! {
                use cynic::CynicError;
                pub async fn query(#arguments) -> Result<#output_type, CynicError<impl std::error::Error + 'static>> {
                    use cynic::QueryBuilder;

                    let query = #name::build(&#struct_definition);
                    let result = crate::gql_schema::run_query(query).await;

                    match result {
                        Ok(result) => {
                            match (result.data, result.errors) {
                                (Some(result), None) => Ok(result.#name),
                                (None, Some(error)) => Err(CynicError::Query(error)),
                                _ => Err(CynicError::Query(vec![]))
                            }
                        },
                        Err(error) => Err(CynicError::Request(error))
                    }
                }
            }
        }
        Definition::Operation(OperationDefinition::Mutation(_)) => {
            quote! {
                use cynic::CynicError;
                pub async fn mutate(#arguments) -> Result<#output_type, CynicError<impl std::error::Error + 'static>> {
                    use cynic::MutationBuilder;

                    let query = #name::build(&#struct_definition);
                    let result = crate::gql_schema::run_query(query).await;

                    match result {
                        Ok(result) => {
                            match (result.data, result.errors) {
                                (Some(result), None) => Ok(result.#name),
                                (None, Some(error)) => Err(CynicError::Query(error)),
                                _ => Err(CynicError::Query(vec![]))
                            }
                        },
                        Err(error) => Err(CynicError::Request(error))
                    }
                }
            }
        }
        Definition::Operation(OperationDefinition::Subscription(_)) => {
            quote! {
                use graphql_ws_client::graphql::Cynic;
                use graphql_ws_client::SubscriptionStream;
                use cynic::StreamingOperation;
                pub async fn subscribe(#arguments) -> Result<SubscriptionStream<Cynic, StreamingOperation<'static, #name>>, impl std::error::Error + 'static> {
                    use cynic::SubscriptionBuilder;

                    let query = #name::build(&#struct_definition);
                    crate::gql_schema::subscribe(query).await
                }
            }
        }
        _ => todo!(),
    };

    let tokens = quote! {
        #[cynic::schema_for_derives(
            file = #schema_path,
            module = "gql_schema",
        )]
        mod #name {
            use crate::gql_schema::{self};
            #(#includes)*
            #(#argument_structs)*
            #(#query_fragments)*
            #(#enums)*
            #(#input_objects)*

            #function
        }
    }
    .into();

    Ok(tokens)
}
