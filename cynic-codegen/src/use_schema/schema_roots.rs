use proc_macro2::TokenStream;

use crate::schema::{types::ObjectType, Definition, Schema, SchemaError, Validated};

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
        let query_name = "Query".to_owned();
        let mutation_name = None;
        let subscription_name = None;

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
            query: schema.lookup::<ObjectType>(&query_name)?,
            mutation: mutation_name
                .map(|name| schema.lookup::<ObjectType>(&name))
                .transpose()?,
            subscription: subscription_name
                .map(|name| schema.lookup::<ObjectType>(&name))
                .transpose()?,
        })
    }
}

impl quote::ToTokens for RootTypes<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = proc_macro2::Ident::from(self.query.marker_ident());

        tokens.append_all(quote! {
            impl ::cynic::schema::QueryRoot for #name {}
        });

        if let Some(mutation) = &self.mutation {
            let name = proc_macro2::Ident::from(mutation.marker_ident());
            tokens.append_all(quote! {
                impl ::cynic::schema::MutationRoot for #name {}
            });
        }

        if let Some(subscription) = &self.subscription {
            let name = proc_macro2::Ident::from(subscription.marker_ident());
            tokens.append_all(quote! {
                impl ::cynic::schema::SubscriptionRoot for #name {}
            });
        }
    }
}
