extern crate proc_macro;

use proc_macro::TokenStream;

use cynic_codegen::{
    enum_derive, fragment_arguments_derive, fragment_derive, inline_fragments_derive,
    input_object_derive, query_dsl, query_module, scalar_derive,
};

#[proc_macro]
pub fn query_dsl(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as query_dsl::QueryDslParams);

    let rv = query_dsl::query_dsl_from_schema(input).unwrap().into();

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
        Err(e) => e.to_compile_error().into(),
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

#[proc_macro_derive(Scalar)]
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
pub fn query_module(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let module = syn::parse_macro_input!(input as syn::ItemMod);
    let attrs = syn::parse_macro_input!(attrs as syn::AttributeArgs);

    let rv: TokenStream = match query_module::transform_query_module(attrs, module) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    };

    // eprintln!("{}", rv);

    rv
}
