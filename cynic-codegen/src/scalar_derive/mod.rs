use proc_macro2::TokenStream;

pub(crate) mod input;

#[cfg(test)]
mod tests;

pub use input::ScalarDeriveInput;
use quote::quote_spanned;

use crate::schema::markers::TypeMarkerIdent;

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

    Ok(quote! {
        #[automatically_derived]
        impl ::cynic::serde::Serialize for #ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::cynic::serde::Serializer,
            {
                self.0.serialize(serializer)
            }
        }

        #[automatically_derived]
        impl<'de> ::cynic::serde::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::cynic::serde::Deserializer<'de>,
            {
                <#inner_type as ::cynic::serde::Deserialize<'de>>::deserialize(deserializer).map(Self)
            }
        }

        #[automatically_derived]
        impl ::cynic::schema::IsScalar<#marker_ident> for #ident {
            type SchemaType = #marker_ident;
        }

        #[automatically_derived]
        impl #schema_module::variable::Variable for #ident {
            const TYPE: ::cynic::variables::VariableType = ::cynic::variables::VariableType::Named(#graphql_type_name);
        }

        ::cynic::impl_coercions!(#ident, #marker_ident);
    })
}
