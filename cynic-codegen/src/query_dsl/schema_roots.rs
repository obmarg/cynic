use proc_macro2::TokenStream;

use super::SelectorStruct;
use crate::{schema, Ident};

#[derive(Debug, PartialEq)]
enum RootType {
    Query,
    Mutation,
}

#[derive(Debug, PartialEq)]
pub struct SchemaRoot {
    name: Ident,
    ty: RootType,
}

impl SchemaRoot {
    pub fn for_query(name: &Ident) -> Self {
        SchemaRoot {
            name: name.clone(),
            ty: RootType::Query,
        }
    }

    pub fn for_mutation(name: &Ident) -> Self {
        SchemaRoot {
            name: name.clone(),
            ty: RootType::Mutation,
        }
    }
}

impl quote::ToTokens for SchemaRoot {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = &self.name;

        tokens.append_all(match self.ty {
            RootType::Mutation => {
                quote! {
                    impl ::cynic::MutationRoot for #name {}
                }
            }
            RootType::Query => {
                quote! {
                    impl ::cynic::QueryRoot for #name {}
                }
            }
        });
    }
}

pub struct RootTypes {
    query: String,
    mutation: String,
}

impl RootTypes {
    pub fn from_definitions(definitions: &[schema::Definition]) -> RootTypes {
        use schema::Definition;

        let mut rv = RootTypes::default();

        for definition in definitions {
            match definition {
                Definition::SchemaDefinition(schema) => {
                    if let Some(query_type) = &schema.query {
                        rv.query = query_type.clone();
                    }
                    if let Some(mutation_type) = &schema.mutation {
                        rv.mutation = mutation_type.clone();
                    }
                    break;
                }
                _ => {}
            }
        }

        return rv;
    }

    pub fn root_from_selector_struct(&self, selector: &SelectorStruct) -> Option<SchemaRoot> {
        if selector.graphql_name == self.query {
            Some(SchemaRoot::for_query(&selector.name))
        } else if selector.graphql_name == self.mutation {
            Some(SchemaRoot::for_mutation(&selector.name))
        } else {
            None
        }
    }
}

impl Default for RootTypes {
    fn default() -> RootTypes {
        RootTypes {
            query: "Query".to_string(),
            mutation: "Mutation".to_string(),
        }
    }
}
