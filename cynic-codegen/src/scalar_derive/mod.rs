use proc_macro2::TokenStream;

pub(crate) mod input;

pub use input::ScalarDeriveInput;

use crate::Ident;

pub fn scalar_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;

    match ScalarDeriveInput::from_derive_input(ast) {
        Ok(input) => scalar_derive_impl(input).or_else(|e| Ok(e.to_compile_error())),
        Err(e) => Ok(e.write_errors()),
    }
}

pub fn scalar_derive_impl(input: ScalarDeriveInput) -> Result<TokenStream, syn::Error> {
    use quote::quote;

    // We're assuming that Darling has already validated this as a newtype enum,
    // so we can get away with panicing here.
    let field = input
        .data
        .take_struct()
        .expect("Expected enum")
        .into_iter()
        .next()
        .expect("Expected enum with one variant");

    let ident = input.ident;
    let inner_type = field.ty;
    let type_lock = if let Some(graphql_type) = input.graphql_type {
        Ident::new_spanned((*graphql_type).clone(), graphql_type.span())
    } else {
        ident.clone().into()
    };

    Ok(quote! {
        impl ::cynic::Scalar<#type_lock> for #ident {
            type Serializable = #inner_type;

            fn from_serializable(inner: Self::Serializable) -> Result<Self, ::cynic::DecodeError> {
                Ok(#ident(inner))
            }

            fn to_serializable(&self) -> Result<&Self::Serializable, ::cynic::SerializeError> {
                Ok(&self.0)
            }
        }

        ::cynic::impl_input_type!(#ident, #type_lock);
    })
}
