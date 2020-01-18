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
            // TODO: Actually not sure I can straight up derive serialize & deserialize
            // as i've probably implemented some transformation on the names.
            // I should update the fields with #serde(from) annotations or something
            #[derive(PartialEq, Eq, Debug, ::serde::Serialize, ::serde::de::DeserializeOwned)]
            pub enum #enum_name {
                #(
                    #enum_values
                ),*
            }

            // TODO: maybe derive the QueryFragment sometime?
            // Also not entirely convinced that this is the right way to do it,
            // should the query fragment code just be enum aware?
            impl QueryFragment for #enum_name {
                type SelectionSet = ::cynic::SelectionSet<'static, Self, ()>;
                type Arguments = ();

                fn selection_set(args: Self::Arguments) -> Self::SelectionSet {
                    ::cynic::selection_set::json()
                }
            }
        })
    }
}
