use proc_macro2::TokenStream;

use crate::{
    schema::{Definition, TypeDefinition},
    Ident,
};

pub struct RootTypes {
    query: String,
    mutation: Option<String>,
    subscription: Option<String>,
}

impl RootTypes {
    pub fn from_definitions(definitions: &[Definition]) -> RootTypes {
        let mut expected_names = RootTypes::default();

        for definition in definitions {
            if let Definition::SchemaDefinition(schema) = definition {
                if let Some(query_type) = &schema.query {
                    expected_names.query = query_type.clone();
                }
                expected_names.mutation = schema.mutation.clone();
                expected_names.subscription = schema.subscription.clone();
                break;
            }
        }

        let mut rv = RootTypes {
            query: expected_names.query,
            mutation: None,
            subscription: None,
        };

        // Now we check that the provided names are present.
        for definition in definitions {
            if let Definition::TypeDefinition(TypeDefinition::Object(obj)) = definition {
                if Some(&obj.name) == expected_names.mutation.as_ref() {
                    rv.mutation = expected_names.mutation.clone();
                }
                if Some(&obj.name) == expected_names.subscription.as_ref() {
                    rv.subscription = expected_names.subscription.clone();
                }
            }
        }

        rv
    }
}

impl quote::ToTokens for RootTypes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = Ident::for_type(&self.query);

        tokens.append_all(quote! {
            impl ::cynic::schema::QueryRoot for #name {}
        });

        if let Some(mutation) = &self.mutation {
            let name = Ident::for_type(mutation);
            tokens.append_all(quote! {
                impl ::cynic::schema::MutationRoot for #name {}
            });
        }

        if let Some(subscription) = &self.subscription {
            let name = Ident::for_type(subscription);
            tokens.append_all(quote! {
                impl ::cynic::schema::SubscriptionRoot for #name {}
            });
        }
    }
}

impl Default for RootTypes {
    fn default() -> RootTypes {
        RootTypes {
            query: "Query".to_string(),
            mutation: Some("Mutation".to_string()),
            subscription: Some("Subscription".to_string()),
        }
    }
}
