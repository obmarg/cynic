use {
    proc_macro2::TokenStream,
    quote::{format_ident, quote},
    syn::visit_mut::{self, VisitMut},
};

mod input;

use crate::generics_for_serde;

use self::input::QueryVariableLiteralsInput;

pub fn query_variable_literals_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;

    match QueryVariableLiteralsInput::from_derive_input(ast) {
        Ok(input) => inlineable_variables_derive_impl(input),
        Err(e) => Ok(e.write_errors()),
    }
}

pub fn inlineable_variables_derive_impl(
    input: QueryVariableLiteralsInput,
) -> Result<TokenStream, syn::Error> {
    let ident = &input.ident;

    let (_, ty_generics, _) = input.generics.split_for_impl();
    let generics_with_ser = generics_for_serde::with_serialize_bounds(&input.generics);
    let (impl_generics_with_ser, _, where_clause_with_ser) = generics_with_ser.split_for_impl();

    let input_fields = input.data.take_struct().unwrap().fields;

    let mut match_arms = Vec::new();

    for f in input_fields {
        let name = f.ident.as_ref().unwrap();

        let name_str =
            proc_macro2::Literal::string(&f.graphql_ident(input.rename_all).graphql_name());

        let mut match_arm_rhs =
            quote! { Some(cynic::queries::to_input_literal(&self.#name).ok()?) };

        if let Some(skip_check_fn) = f.skip_serializing_if {
            let skip_check_fn = &*skip_check_fn;
            match_arm_rhs = quote! {
                if #skip_check_fn(&self.#name) {
                    None
                } else {
                    #match_arm_rhs
                }
            }
        }

        match_arms.push(quote! {
            #name_str => #match_arm_rhs,
        })
    }

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics_with_ser cynic::QueryVariableLiterals for #ident #ty_generics #where_clause_with_ser {
            fn get(&self, variable_name: &str) -> Option<cynic::queries::InputLiteral> {
                match variable_name {
                    #(#match_arms)*
                    _ => None
                }
            }
        }
    })
}

struct TurnLifetimesToStatic;
impl VisitMut for TurnLifetimesToStatic {
    fn visit_lifetime_mut(&mut self, i: &mut syn::Lifetime) {
        i.ident = format_ident!("static");
        visit_mut::visit_lifetime_mut(self, i)
    }
}
