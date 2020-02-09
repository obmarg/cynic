use proc_macro2::TokenStream;

use crate::Ident;

/// We generate a UnionStruct for each union in the schema.
///
/// These are output as empty structs that can be used as the TypeLock
/// in a SelectorStruct.
#[derive(Debug)]
pub struct UnionStruct {
    pub name: Ident,
    pub subtypes: Vec<Ident>,
}

impl UnionStruct {
    pub fn from_union(union: &graphql_parser::schema::UnionType) -> Self {
        UnionStruct {
            name: Ident::for_type(&union.name),
            subtypes: union.types.iter().map(|ty| Ident::for_type(ty)).collect(),
        }
    }
}

impl quote::ToTokens for UnionStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = &self.name;
        let subtypes = &self.subtypes;

        tokens.append_all(quote! {
            pub struct #name {}

            #(
                impl ::cynic::selection_set::HasSubtype<#subtypes> for #name {}
            )*
        });
    }
}
