extern crate proc_macro;

use proc_macro::TokenStream;

mod error;
mod fragment_derive;
mod ident;
mod module;
mod query_dsl;

use error::Error;
use ident::Ident;

#[proc_macro]
pub fn query_dsl(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as query_dsl::QueryDslParams);

    query_dsl::query_dsl_from_schema(input).unwrap().into()
}

#[proc_macro_derive(QueryFragment, attributes(cynic))]
pub fn query_fragment_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    match fragment_derive::fragment_derive(&ast) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
