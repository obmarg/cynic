use graphql_parser::schema::EnumType;
use proc_macro2::TokenStream;

use crate::ident::Ident;

#[derive(Debug)]
pub struct GraphQLEnum {
    name: Ident,
    value_names: Vec<Ident>,
}

impl From<EnumType> for GraphQLEnum {
    fn from(enum_type: EnumType) -> Self {
        GraphQLEnum {
            name: Ident::for_type(&enum_type.name),
            value_names: enum_type
                .values
                .iter()
                .map(|v| Ident::for_type(&v.name))
                .collect(),
        }
    }
}

impl quote::ToTokens for GraphQLEnum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let enum_name = &self.name;
        let enum_values = &self.value_names;

        tokens.append_all(quote! {
            #[derive(PartialEq, Eq, Debug, ::serde::Serialize, ::serde::de::DeserializeOwned)]
            pub enum #enum_name {
                #(
                    #enum_values
                ),*
            }
        })
    }
}
