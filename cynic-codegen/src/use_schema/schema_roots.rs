use proc_macro2::TokenStream;

use crate::schema::types::SchemaRoots;

impl quote::ToTokens for SchemaRoots<'_> {
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
