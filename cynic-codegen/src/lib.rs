extern crate proc_macro;

use proc_macro::TokenStream;

mod attributes;
mod error;
mod field_type;
mod fragment_arguments_derive;
mod fragment_derive;
mod graphql_extensions;
mod ident;
mod inline_fragments_derive;
mod module;
mod query_dsl;
mod struct_field;
mod type_index;
mod type_path;

use error::Error;
use field_type::FieldType;
use ident::Ident;
use struct_field::StructField;
use type_index::TypeIndex;
use type_path::TypePath;

#[proc_macro]
pub fn query_dsl(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as query_dsl::QueryDslParams);

    let rv = query_dsl::query_dsl_from_schema(input).unwrap().into();

    rv
}

#[proc_macro_derive(QueryFragment, attributes(cynic, cynic_arguments))]
pub fn query_fragment_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let rv = match fragment_derive::fragment_derive(&ast) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    };

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
        Err(e) => e.to_compile_error().into(),
    };

    rv
}
