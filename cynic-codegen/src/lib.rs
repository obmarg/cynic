extern crate proc_macro;

use proc_macro::TokenStream;

mod error;
mod field_type;
mod fragment_derive;
mod graphql_extensions;
mod ident;
mod module;
mod query_dsl;
mod struct_field;
mod type_path;

use error::Error;
use field_type::FieldType;
use ident::Ident;
use struct_field::StructField;
use type_path::TypePath;

#[proc_macro]
pub fn query_dsl(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as query_dsl::QueryDslParams);

    let rv = query_dsl::query_dsl_from_schema(input).unwrap().into();

    eprintln!("{}", rv);
    rv
}

#[proc_macro_derive(QueryFragment, attributes(cynic, cynic_arguments))]
pub fn query_fragment_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let rv = match fragment_derive::fragment_derive(&ast) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    };

    eprintln!("{}", rv);
    rv
}
