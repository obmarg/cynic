use proc_macro2::TokenStream;
use quote::{format_ident, quote};

mod input;

use self::input::QueryVariablesDeriveInput;

// TODO: Rename this derive to QueryVariables and provide deprecation
// warnings if the old one is used...
pub fn query_variables_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;

    match QueryVariablesDeriveInput::from_derive_input(ast) {
        Ok(input) => query_variables_derive_impl(input),
        Err(e) => Ok(e.write_errors()),
    }
}

pub fn query_variables_derive_impl(
    input: QueryVariablesDeriveInput,
) -> Result<TokenStream, syn::Error> {
    let ident = &input.ident;
    let vis = &input.vis;
    let schema_module = &input.schema_module();
    let fields_struct_ident = format_ident!("{}Fields", ident);

    let input_fields = input.data.take_struct().unwrap().fields;

    let mut field_funcs = Vec::new();
    let mut variables = Vec::new();
    let mut field_inserts = Vec::new();

    for f in input_fields {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        let name_str =
            proc_macro2::Literal::string(&f.graphql_ident(input.rename_all).graphql_name());

        field_funcs.push(quote! {
            #vis fn #name() -> ::cynic::variables::VariableDefinition<#ident, #ty> {
                ::cynic::variables::VariableDefinition::new(#name_str)
            }
        });

        variables.push(quote! {
            (#name_str, <#ty as #schema_module::variable::Variable>::TYPE)
        });

        field_inserts.push(quote! {
            map_serializer.serialize_entry(#name_str, &self.#name);
        })
    }

    let map_len = field_inserts.len();

    Ok(quote! {

        #[automatically_derived]
        impl ::cynic::QueryVariables for #ident {
            type Fields = #fields_struct_ident;

            const VARIABLES: &'static [(&'static str, ::cynic::variables::VariableType)]
                = &[#(#variables),*];
        }

        #[automatically_derived]
        impl ::cynic::serde::Serialize for #ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::cynic::serde::Serializer,
            {
                use ::cynic::serde::ser::SerializeMap;

                let mut map_serializer = serializer.serialize_map(Some(#map_len))?;

                #(#field_inserts)*

                map_serializer.end()
            }
        }

        impl ::cynic::queries::VariableMatch<#ident> for #ident {}

        #vis struct #fields_struct_ident;

        impl #fields_struct_ident {
            #(
                #field_funcs
            )*
        }
    })
}
