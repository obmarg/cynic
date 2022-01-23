use proc_macro2::TokenStream;

pub struct DeserializeImpl<'a> {
    pub(super) target_enum: syn::Ident,
    pub(super) fragments: &'a [super::Fragment],
    pub(super) fallback: Option<(syn::Ident, Option<syn::Type>)>,
}

impl quote::ToTokens for DeserializeImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let target_enum = &self.target_enum;
        let inner_types = self.fragments.iter().map(|fragment| &fragment.inner_type);
        let graphql_types = self
            .fragments
            .iter()
            .map(|fragment| proc_macro2::Literal::string(&fragment.graphql_type));
        let variant_names = self
            .fragments
            .iter()
            .map(|fragment| &fragment.rust_variant_name)
            .collect::<Vec<_>>();

        let (intermediate_fallback_def, fallback_match) = match &self.fallback {
            Some((fallback_variant, Some(fallback_inner))) => (
                Some(quote! { #[serde(other)] Fallback(#fallback_inner)}),
                Some(
                    quote! { Intermediate::Fallback(inner) => #target_enum::#fallback_variant(inner) },
                ),
            ),
            Some((fallback_variant, None)) => (
                Some(quote! { #[serde(other)] Fallback }),
                Some(quote! { Intermediate::Fallback => #target_enum::#fallback_variant }),
            ),
            None => (None, None),
        };

        tokens.append_all(quote! {
            #[automatically_derived]
            impl<'de> ::cynic::serde::Deserialize<'de> for #target_enum {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: ::cynic::serde::Deserializer<'de>,
                {
                    #[derive(::cynic::serde::Deserialize)]
                    #[serde(tag = "__typename", crate="::cynic::serde")]
                    enum Intermediate {
                        #(
                            #[serde(rename = #graphql_types)]
                            #variant_names(#inner_types),
                        )*
                        #intermediate_fallback_def
                    }

                    Ok(match Intermediate::deserialize(deserializer)? {
                        #(
                            Intermediate::#variant_names(inner) => #target_enum::#variant_names(inner),
                        )*
                        #fallback_match
                    })
                }
            }
        });
    }
}
