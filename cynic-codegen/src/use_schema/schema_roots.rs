use proc_macro2::TokenStream;

use crate::schema::{parser::Definition, types::ObjectType, Schema, SchemaError, Validated};

pub struct RootTypes<'a> {
    query: ObjectType<'a>,
    mutation: Option<ObjectType<'a>>,
    subscription: Option<ObjectType<'a>>,
}

impl<'a> RootTypes<'a> {
    pub fn from_definitions(
        definitions: &[Definition],
        schema: &Schema<'a, Validated>,
    ) -> Result<RootTypes<'a>, SchemaError> {
        let mut query_name = "Query".to_owned();
        let mut mutation_name = Some("Mutation".to_owned());
        let mut subscription_name = Some("Subscription".to_owned());

        for definition in definitions {
            if let Definition::SchemaDefinition(schema) = definition {
                if let Some(query_type) = &schema.query {
                    query_name = query_type.clone();
                }
                mutation_name = schema.mutation.clone();
                subscription_name = schema.subscription.clone();
                break;
            }
        }

        Ok(RootTypes {
            query: schema.lookup::<ObjectType<'_>>(&query_name)?,
            mutation: mutation_name
                .and_then(|name| schema.try_lookup::<ObjectType<'_>>(&name).ok()),
            subscription: subscription_name
                .and_then(|name| schema.try_lookup::<ObjectType<'_>>(&name).ok()),
        })
    }
}

impl quote::ToTokens for RootTypes<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = self.query.marker_ident().to_rust_ident();

        tokens.append_all(quote! {
            impl cynic::schema::QueryRoot for #name {}
        });

        if let Some(mutation) = &self.mutation {
            let name = mutation.marker_ident().to_rust_ident();
            tokens.append_all(quote! {
                impl cynic::schema::MutationRoot for #name {}
            });
        }

        if let Some(subscription) = &self.subscription {
            let name = subscription.marker_ident().to_rust_ident();
            tokens.append_all(quote! {
                impl cynic::schema::SubscriptionRoot for #name {}
            });
        }
    }
}
