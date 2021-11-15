//! Procedural macros for cynic

#![allow(clippy::let_and_return)]
#![warn(missing_docs)]

extern crate proc_macro;

use proc_macro::TokenStream;

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
