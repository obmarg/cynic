use graphql_parser::schema::EnumType;
use proc_macro2::TokenStream;

use crate::ident::Ident;

#[derive(Debug)]
pub struct GraphQLEnum {
    name: Ident,
    rust_value_names: Vec<Ident>,
    graphql_names: Vec<String>,
}

impl From<EnumType> for GraphQLEnum {
    fn from(enum_type: EnumType) -> Self {
        GraphQLEnum {
            name: Ident::for_type(&enum_type.name),
            rust_value_names: enum_type
                .values
                .iter()
                .map(|v| Ident::for_type(&v.name))
                .collect(),
            graphql_names: enum_type.values.iter().map(|v| v.name.clone()).collect(),
        }
    }
}

impl quote::ToTokens for GraphQLEnum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let enum_name = &self.name;
        let enum_values = &self.rust_value_names;
        let graphql_names = self
            .graphql_names
            .iter()
            .map(|n| proc_macro2::Literal::string(&n));
        let string_enum_name = proc_macro2::Literal::string(&enum_name.to_string());

        tokens.append_all(quote! {
            #[derive(PartialEq, Eq, Debug, ::serde::Serialize, ::serde::Deserialize)]
            pub enum #enum_name {
                #(
                    #[serde(rename=#graphql_names)]
                    #enum_values
                ),*
            }

            // TODO: maybe derive the QueryFragment sometime?
            // Also not entirely convinced that this is the right way to do it,
            // should the query fragment code just be enum aware?
            impl ::cynic::QueryFragment for #enum_name {
                type SelectionSet = ::cynic::SelectionSet<'static, Self, ()>;
                type Arguments = ();

                fn fragment(args: Self::Arguments) -> Self::SelectionSet {
                    ::cynic::selection_set::serde()
                }

                fn graphql_type() -> String {
                    #string_enum_name.to_string()
                }
            }
        })
    }
}
