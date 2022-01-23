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
    let fields_struct_ident = format_ident!("{}Fields", ident);

    let input_fields = input.data.take_struct().unwrap().fields;
    let fields_funcs = input_fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        let name_str =
            proc_macro2::Literal::string(&f.graphql_ident(input.rename_all).graphql_name());

        quote! {
            #vis fn #name() -> ::cynic::core::VariableDefinition<#ident, #ty> {
                ::cynic::core::VariableDefinition::new(#name_str)
            }
        }
    });

    Ok(quote! {
        impl ::cynic::core::QueryVariables for #ident {
            type Fields = #fields_struct_ident;
        }

        #vis struct #fields_struct_ident;

        impl #fields_struct_ident {
            #(
                #fields_funcs
            )*
        }
    })
}
