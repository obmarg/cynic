use proc_macro2::TokenStream;
use quote::{format_ident, quote};

mod input;

use self::input::FragmentArgumentsDeriveInput;

// TODO: Rename this derive to QueryVariables and provide deprecation
// warnings if the old one is used...
pub fn fragment_arguments_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;

    match FragmentArgumentsDeriveInput::from_derive_input(ast) {
        Ok(input) => fragment_arguments_derive_impl(input),
        Err(e) => Ok(e.write_errors()),
    }
}

pub fn fragment_arguments_derive_impl(
    input: FragmentArgumentsDeriveInput,
) -> Result<TokenStream, syn::Error> {
    let ident = &input.ident;
    let vis = &input.vis;
    let schema_module = &input.schema_module();
    let fields_struct_ident = format_ident!("{}Fields", ident);

    let input_fields = input.data.take_struct().unwrap().fields;

    let mut field_funcs = Vec::new();
    let mut variables = Vec::new();

    for f in input_fields {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        let name_str =
            proc_macro2::Literal::string(&f.graphql_ident(input.rename_all).graphql_name());

        field_funcs.push(quote! {
            #vis fn #name() -> ::cynic::core::VariableDefinition<#ident, #ty> {
                ::cynic::core::VariableDefinition::new(#name_str)
            }
        });

        variables.push(quote! {
            (#name_str, <#ty as #schema_module::variable::Variable>::TYPE)
        });
    }

    Ok(quote! {
        impl ::cynic::core::QueryVariables for #ident {
            type Fields = #fields_struct_ident;

            const VARIABLES: &'static [(&'static str, ::cynic::core::VariableType)]
                = &[#(#variables),*];
        }

        #vis struct #fields_struct_ident;

        impl #fields_struct_ident {
            #(
                #field_funcs
            )*
        }
    })
}
