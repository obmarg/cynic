use proc_macro2::TokenStream;
use std::collections::VecDeque;

use crate::ident::Ident;

/// The path to a type e.g. serde_json::Value
///
/// Implements ToToken so it can be used inside quote!
#[derive(Debug)]
pub struct TypePath {
    path: VecDeque<Ident>,
}

impl TypePath {
    pub fn new(path: Vec<Ident>) -> Self {
        TypePath { path: path.into() }
    }
}

impl From<Ident> for TypePath {
    fn from(ident: Ident) -> TypePath {
        TypePath {
            path: vec![ident].into(),
        }
    }
}

impl quote::ToTokens for TypePath {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};
        use std::iter::FromIterator;

        let path = self.path.iter();

        tokens.append_all(quote! {
             #(
                 #path
             )::*
        })
    }
}
