use proc_macro2::TokenStream;

pub(crate) mod input;

#[cfg(test)]
mod tests;

pub use input::ScalarDeriveInput;
use quote::quote_spanned;

use crate::{generics_for_serde, schema::markers::TypeMarkerIdent};

pub fn scalar_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;

    match ScalarDeriveInput::from_derive_input(ast) {
        Ok(input) => scalar_derive_impl(input).or_else(|e| Ok(e.to_compile_error())),
        Err(e) => Ok(e.write_errors()),
    }
}

pub fn scalar_derive_impl(input: ScalarDeriveInput) -> Result<TokenStream, syn::Error> {
    use quote::quote;

    let schema_module = input.schema_module();

    // We're assuming that Darling has already validated this as a newtype enum,
    // so we can get away with panicking here.
    let field = input
        .data
        .take_struct()
        .expect("Expected enum")
        .into_iter()
        .next()
        .expect("Expected enum with one variant");

    let ident = input.ident;
    let inner_type = field.ty;

    let scalar_name;
    let scalar_span;
    match input.graphql_type {
        Some(graphql_type) => {
            scalar_span = graphql_type.span();
            scalar_name = graphql_type.to_string();
        }
        None => {
            scalar_span = ident.span();
            scalar_name = ident.to_string();
        }
    }

    let graphql_type_name = proc_macro2::Literal::string(&scalar_name);
    let marker_ident = TypeMarkerIdent::with_graphql_name(&scalar_name).to_path(&schema_module);
    let marker_ident = quote_spanned! { scalar_span => #marker_ident };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let generics_with_de = generics_for_serde::with_de_and_deserialize_bounds(&input.generics);
    let (impl_generics_with_de, _, where_clause_with_de) = generics_with_de.split_for_impl();
    let generics_with_ser = generics_for_serde::with_serialize_bounds(&input.generics);
    let (impl_generics_with_ser, _, where_clause_with_ser) = generics_with_ser.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics_with_ser cynic::serde::Serialize for #ident #ty_generics #where_clause_with_ser {
            fn serialize<__S>(&self, serializer: __S) -> Result<__S::Ok, __S::Error>
            where
                __S: cynic::serde::Serializer,
            {
                <#inner_type as cynic::serde::Serialize>::serialize(&self.0, serializer)
            }
        }

        #[automatically_derived]
        impl #impl_generics_with_de cynic::serde::Deserialize<'de> for #ident #ty_generics #where_clause_with_de {
            fn deserialize<__D>(deserializer: __D) -> Result<Self, __D::Error>
            where
                __D: cynic::serde::Deserializer<'de>,
            {
                <#inner_type as cynic::serde::Deserialize<'de>>::deserialize(deserializer).map(Self)
            }
        }

        #[automatically_derived]
        impl #impl_generics cynic::schema::IsScalar<#marker_ident> for #ident #ty_generics #where_clause {
            type SchemaType = #marker_ident;
        }

        #[automatically_derived]
        impl #impl_generics #schema_module::variable::Variable for #ident #ty_generics #where_clause {
            const TYPE: cynic::variables::VariableType = cynic::variables::VariableType::Named(#graphql_type_name);
        }

        cynic::impl_coercions!(#ident #ty_generics [#impl_generics] [#where_clause], #marker_ident);
    })
}
