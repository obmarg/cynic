use {
    proc_macro2::TokenStream,
    quote::{format_ident, quote, quote_spanned},
    syn::visit_mut::{self, VisitMut},
};

mod input;

use crate::variables_fields_ident;

use self::input::QueryVariablesDeriveInput;

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
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let vis = &input.vis;
    let schema_module = &input.schema_module();
    let fields_struct_ident = variables_fields_ident(ident);

    let input_fields = input.data.take_struct().unwrap().fields;

    let mut field_funcs = Vec::new();
    let mut variables = Vec::new();
    let mut field_inserts = Vec::new();

    for f in input_fields {
        let name = f.ident.as_ref().unwrap();
        let mut ty = f.ty.clone();
        TurnLifetimesToStatic.visit_type_mut(&mut ty);
        let name_str =
            proc_macro2::Literal::string(&f.graphql_ident(input.rename_all).graphql_name());

        field_funcs.push(quote! {
            #vis fn #name() -> ::cynic::variables::VariableDefinition<Self, #ty> {
                ::cynic::variables::VariableDefinition::new(#name_str)
            }
        });

        variables.push(quote! {
            (#name_str, <#ty as #schema_module::variable::Variable>::TYPE)
        });

        field_inserts.push(quote! {
            map_serializer.serialize_entry(#name_str, &self.#name)?;
        })
    }

    let map_len = field_inserts.len();

    let ident_span = ident.span();
    let fields_struct = quote_spanned! { ident_span =>
        #vis struct #fields_struct_ident;

        impl ::cynic::QueryVariablesFields for #fields_struct_ident {
            const VARIABLES: &'static [(&'static str, ::cynic::variables::VariableType)]
                = &[#(#variables),*];
        }

        impl ::cynic::queries::VariableMatch<#fields_struct_ident> for #fields_struct_ident {}

        impl #fields_struct_ident {
            #(
                #field_funcs
            )*
        }
    };

    Ok(quote! {

        #[automatically_derived]
        impl #impl_generics ::cynic::QueryVariables for #ident #ty_generics #where_clause {
            type Fields = #fields_struct_ident;
        }

        #[automatically_derived]
        impl #impl_generics ::cynic::serde::Serialize for #ident #ty_generics #where_clause {
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

        #fields_struct
    })
}

struct TurnLifetimesToStatic;
impl VisitMut for TurnLifetimesToStatic {
    fn visit_lifetime_mut(&mut self, i: &mut syn::Lifetime) {
        i.ident = format_ident!("static");
        visit_mut::visit_lifetime_mut(self, i)
    }
}
