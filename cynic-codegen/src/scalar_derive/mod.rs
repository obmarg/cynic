use darling::util::SpannedValue;
use graphql_parser::schema::{Definition, Document, EnumType, EnumValue, TypeDefinition};
use proc_macro2::{Span, TokenStream};
use std::collections::{HashMap, HashSet};

use crate::{
    ident::{RenameAll, RenameRule},
    load_schema, Ident,
};

pub(crate) mod input;

use input::ScalarDeriveField;
pub use input::ScalarDeriveInput;

pub fn scalar_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;

    match ScalarDeriveInput::from_derive_input(ast) {
        Ok(input) => scalar_derive_impl(input).or_else(|e| Ok(e.to_compile_error())),
        Err(e) => Ok(e.write_errors()),
    }
}

pub fn scalar_derive_impl(input: ScalarDeriveInput) -> Result<TokenStream, syn::Error> {
    use quote::{quote, quote_spanned};

    // We're assuming that Darling has already validated this as a newtype enum,
    // so we can get away with panicing here.
    let field = input
        .data
        .take_struct()
        .expect("Expected enum")
        .into_iter()
        .nth(0)
        .expect("Expected enum with one variant");

    let ident = input.ident;
    let inner_type = field.ty;

    Ok(quote! {
        impl ::cynic::Scalar for #ident {
            fn decode(value: &serde_json::Value) -> Result<Self, ::cynic::DecodeError> {
                Ok(#ident(<#inner_type as ::cynic::Scalar>::decode(value)?))
            }
        }
    })
}
