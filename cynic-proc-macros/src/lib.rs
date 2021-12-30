//! Procedural macros for cynic

#![allow(clippy::let_and_return)]
#![warn(missing_docs)]
#![feature(proc_macro_span, proc_macro_diagnostic)]

extern crate proc_macro;

use std::io::{Read, Write};
use std::ops::Range;
use std::process::{Command, Stdio};
use std::{path::Path, rc::Rc};

use cargo_toml::Manifest;
use cynic_querygen::{parse_query_document, Error, TypeIndex};
use graphql_parser::error::Error as ConsumeError;
use graphql_parser::query::{Definition, OperationDefinition};
use proc_macro::{Delimiter, Span, TokenStream};
use proc_macro2::Ident;
use quote::quote;
use serde::Deserialize;

use cynic_codegen::{
    enum_derive, fragment_arguments_derive, fragment_derive, inline_fragments_derive,
    input_object_derive, scalar_derive, schema_for_derives, use_schema,
};
use cynic_querygen::ArgumentStructField;

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

/// Derives `cynic::QueryFragment`
///
/// See [the book for usage details](https://cynic-rs.dev/derives/query-fragments.html)
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

/// Derives `cynic::FragmentArguments`
///
/// See [the book for usage details](https://cynic-rs.dev/derives/query-fragments.html#passing-arguments)
#[proc_macro_derive(FragmentArguments)]
pub fn fragment_arguments_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let rv = match fragment_arguments_derive::fragment_arguments_derive(&ast) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    };

    rv
}

/// Derives `cynic::InlineFragments`
///
/// See [the book for usage details](https://cynic-rs.dev/derives/inline-fragments.html)
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

/// Derives `cynic::Enum`
///
/// See [the book for usage details](https://cynic-rs.dev/derives/enums.html)
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

/// Derives `cynic::Scalar`
///
/// See [the book for usage details](https://cynic-rs.dev/derives/scalars.html)
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

/// Derives `InputObject`
///
/// See [the book for usage details](https://cynic-rs.dev/derives/input-objects.html)
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

/// An attribute macro to automatically add schema attributes to cynic derives.
///
/// Most cynic derives take `schema_path` & `query_module` parameters and adding each
/// of these to every derive can be laborious and verbose.  This attribute provides
/// a way to avoid repeating yourself quite as much.
///
/// See [the book for usage details](https://cynic-rs.dev/derives/schema-for-derives.html)
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

/// Compiles a source string of all input tokens with one space between all tokens while storing the token spans.
/// This is requires so we can later map error messages to graphql-parser errors with [`span_in_stream`].
fn catalog_tokens(
    input: TokenStream,
    catalog: &mut Vec<(Range<usize>, Span)>,
    source_string: &mut String,
) {
    for token in input {
        use proc_macro::TokenTree;

        match token {
            TokenTree::Group(ref t) => {
                let previous_length = source_string.len();
                source_string.push_str(match t.delimiter() {
                    Delimiter::Parenthesis => "( ",
                    Delimiter::Brace => "{ ",
                    Delimiter::Bracket => "[ ",
                    Delimiter::None => " ",
                });
                catalog.push((previous_length..source_string.len() - 1, t.span()));

                catalog_tokens(t.stream(), catalog, source_string);

                let previous_length = source_string.len();
                source_string.push_str(match t.delimiter() {
                    Delimiter::Parenthesis => ") ",
                    Delimiter::Brace => "} ",
                    Delimiter::Bracket => "] ",
                    Delimiter::None => " ",
                });
                catalog.push((previous_length..source_string.len() - 1, t.span()));
            }
            _ => {
                let previous_length = source_string.len();
                source_string.push_str(&format!("{} ", token));
                catalog.push((previous_length..source_string.len() - 1, token.span()));
            }
        }
    }
}

/// Gets the rustc [`Span`] for a given span of the graphql-parser in the form of a position and a length.
fn full_span_in_stream(
    catalog: &Vec<(Range<usize>, Span)>,
    position: usize,
    len: usize,
) -> Option<Span> {
    // The graphql parser starts position numbers at 1 instead of 0.
    let position = position - 1;
    let mut start_span = None;
    let mut end_span = None;
    for token in catalog {
        if token.0.contains(&position) {
            start_span = Some(token.1);
        }

        if token.0.contains(&(position + len - 1)) {
            end_span = Some(token.1);
        }
    }

    if let (Some(start_span), Some(end_span)) = (start_span, end_span) {
        start_span.join(end_span)
    } else {
        None
    }
}

/// Gets the rustc [`Span`] for a given span of the graphql-parser in the form of a position.
/// This should only be used as a fallback for when run in rust-analyzer where joining Spans does not work yet.
/// Try using [`full_span_in_stream`] or fall back to this if [`full_span_in_stream`] returns `None`.
fn span_in_stream(catalog: &Vec<(Range<usize>, Span)>, position: usize) -> Option<Span> {
    // The graphql parser starts position numbers at 1 instead of 0.
    let position = position - 1;
    for token in catalog {
        if token.0.contains(&position) {
            return Some(token.1);
        }
    }

    None
}

/// This macro provides a convenient way to build GraphQL queries, mutations and subscriptions on the client side.
///
/// Example
/// ```
/// cynic::gql! {
/// mutation create_user($username: String!, $password: String!) {
///     createUser(input: { username: $username, password: $password })
/// }
/// ```
#[proc_macro]
pub fn gql(input: TokenStream) -> TokenStream {
    // eprintln!("============================================================");
    let package_root = std::env::var("CARGO_MANIFEST_DIR")
        .expect("You are trying to call this from outside of cargo.");
    let path = extract_schema(&package_root)
        .expect("No schema path found in the Cargo.toml. Use `metadata.cynic.schema`.");
    let schema = std::fs::read_to_string(Path::new(&package_root).join(&path))
        .expect("Schema could not be read.");

    let mut catalog = vec![];
    let mut source = String::new();
    catalog_tokens(input.clone(), &mut catalog, &mut source);

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

                if let Some(span) = full_span_in_stream(&catalog, error.position.column, 1) {
                    span.error(error_message).emit();
                } else if let Some(_) = span_in_stream(&catalog, error.position.column) {
                    panic!("{}", error_message);
                } else {
                    panic!("Failed to get diagnostics. Get more info with `cargo build`.");
                }

                return quote! {}.into();
            }
            Error::UnknownField(field, object, span) => {
                if let Some(span) = full_span_in_stream(&catalog, span.column, field.len()) {
                    span.error(format!(
                        "`{}` here was not found on the `{}` object.",
                        field, object
                    ))
                    .emit()
                } else if let Some(_) = span_in_stream(&catalog, span.column) {
                    panic!("`{}` here was not found on the `{}` object.", field, object);
                } else {
                    panic!("Failed to get diagnostics. Get more info with `cargo build`.");
                }

                return quote! {}.into();
            }
            Error::ArgumentNotEnum(field, span) => {
                if let Some(span) = full_span_in_stream(&catalog, span.column, field.len()) {
                    span.error(format!(
                        "`{}` here was used like an enum but is no enum.",
                        field
                    ))
                    .emit()
                } else if let Some(_) = span_in_stream(&catalog, span.column) {
                    panic!("`{}` here was used like an enum but is no enum.", field);
                } else {
                    panic!("Failed to get diagnostics. Get more info with `cargo build`.");
                }

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

            let mut fields = argument_struct.fields.clone();
            fields.sort_by(|a, b| match (a, b) {
                (ArgumentStructField::Variable(a), ArgumentStructField::Variable(b)) => a
                    .name
                    .1
                    .line
                    .cmp(&b.name.1.line)
                    .then(a.name.1.column.cmp(&b.name.1.column)),
                _ => unimplemented!(),
            });

            for field in &fields {
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
