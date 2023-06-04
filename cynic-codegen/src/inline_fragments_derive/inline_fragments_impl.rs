use {proc_macro2::TokenStream, quote::quote_spanned, syn::spanned::Spanned};

use crate::generics_for_serde;

#[derive(Clone)]
pub enum Fallback {
    UnionUnitVariant(syn::Ident),
    UnionVariantWithTypename(syn::Ident, syn::Type),
    InterfaceVariant(syn::Ident, syn::Type),
}

pub struct InlineFragmentsImpl<'a> {
    pub(super) target_enum: syn::Ident,
    pub(super) fragments: &'a [super::Fragment],
    pub(super) fallback: Option<Fallback>,
    pub(super) generics: &'a syn::Generics,
}

impl quote::ToTokens for InlineFragmentsImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let target_enum = &self.target_enum;
        let inner_types = self.fragments.iter().map(|fragment| &fragment.inner_type);
        let variant_names = self
            .fragments
            .iter()
            .map(|fragment| &fragment.rust_variant_name)
            .collect::<Vec<_>>();

        let (_, ty_generics, _) = self.generics.split_for_impl();
        let generics_with_de = generics_for_serde::with_de_and_deserialize_bounds(self.generics);
        let (impl_generics_with_de, _, where_clause_with_de) = generics_with_de.split_for_impl();

        let fallback = match &self.fallback {
            Some(Fallback::UnionUnitVariant(variant)) => quote! {
                Ok(#target_enum::#variant)
            },
            Some(Fallback::UnionVariantWithTypename(variant, ty)) => {
                let ty_span = ty.span();
                quote_spanned! { ty_span => {
                        cynic::assert_type_eq_all!(#ty, String);
                        Ok(#target_enum::#variant(typename.to_string()))
                    }
                }
            }
            Some(Fallback::InterfaceVariant(variant, ty)) => quote! {
                <#ty as cynic::serde::Deserialize<'de>>::deserialize(deserializer).map(
                    #target_enum::#variant
                )
            },
            None => {
                quote! {
                    use cynic::serde::de::Error;
                    Err(D::Error::custom(format!("Unknown type: {}", typename)))
                }
            }
        };

        tokens.append_all(quote! {
            #[automatically_derived]
            impl #impl_generics_with_de cynic::serde::Deserialize<'de> for #target_enum #ty_generics #where_clause_with_de {
                fn deserialize<__D>(deserializer: __D) -> Result<Self, __D::Error>
                where
                    __D: cynic::serde::Deserializer<'de>,
                {
                    deserializer.deserialize_map(cynic::__private::InlineFragmentVisitor::<Self>::new())
                }
            }

            #[automatically_derived]
            impl #impl_generics_with_de cynic::InlineFragments<'de> for #target_enum #ty_generics #where_clause_with_de {
                fn deserialize_variant<__D>(typename: &str, deserializer: __D) -> Result<Self, __D::Error>
                where
                    __D: cynic::serde::Deserializer<'de>
                {
                    #(
                        if Some(typename) == <#inner_types as cynic::QueryFragment>::TYPE {
                            return <#inner_types as cynic::serde::Deserialize<'de>>::deserialize(deserializer).map(
                                #target_enum::#variant_names
                            )
                        }
                    )*

                    #fallback
                }
            }
        });
    }
}
