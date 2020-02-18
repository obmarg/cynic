extern crate proc_macro;

use proc_macro::TokenStream;

use cynic_codegen::{
    fragment_arguments_derive, fragment_derive, inline_fragments_derive, query_dsl,
    scalars_as_strings,
};

#[proc_macro]
pub fn query_dsl(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as query_dsl::QueryDslParams);

    let rv = query_dsl::query_dsl_from_schema(input).unwrap().into();

    //eprintln!("{}", rv);

    rv
}

#[proc_macro]
pub fn scalars_as_strings(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as scalars_as_strings::Params);

    let rv = scalars_as_strings::scalars_as_strings(input)
        .unwrap()
        .into();

    //eprintln!("{}", rv);

    rv
}

#[proc_macro_derive(QueryFragment, attributes(cynic, cynic_arguments))]
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
        Err(e) => e.to_compile_error().into(),
    };

    //eprintln!("{}", rv);

    rv
}
