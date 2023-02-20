use {proc_macro2::TokenStream, quote::quote_spanned, syn::spanned::Spanned};

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

        let fallback = match &self.fallback {
            Some(Fallback::UnionUnitVariant(variant)) => quote! {
                Ok(#target_enum::#variant)
            },
            Some(Fallback::UnionVariantWithTypename(variant, ty)) => {
                let ty_span = ty.span();
                quote_spanned! { ty_span => {
                        ::cynic::assert_type_eq_all!(#ty, String);
                        Ok(#target_enum::#variant(typename.to_string()))
                    }
                }
            }
            Some(Fallback::InterfaceVariant(variant, ty)) => quote! {
                <#ty as ::cynic::serde::Deserialize<'de>>::deserialize(deserializer).map(
                    #target_enum::#variant
                )
            },
            None => {
                quote! {
                    use ::cynic::serde::de::Error;
                    Err(D::Error::custom(format!("Unknown type: {}", typename)))
                }
            }
        };

        tokens.append_all(quote! {
            #[automatically_derived]
            impl<'de> ::cynic::serde::Deserialize<'de> for #target_enum {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: ::cynic::serde::Deserializer<'de>,
                {
                    deserializer.deserialize_map(::cynic::__private::InlineFragmentVisitor::<Self>::new())
                }
            }

            #[automatically_derived]
            impl<'de> ::cynic::InlineFragments<'de> for #target_enum {
                fn deserialize_variant<D>(typename: &str, deserializer: D) -> Result<Self, D::Error>
                where
                    D: ::cynic::serde::Deserializer<'de>
                {
                    #(
                        if Some(typename) == <#inner_types as ::cynic::QueryFragment>::TYPE {
                            return <#inner_types as ::cynic::serde::Deserialize<'de>>::deserialize(deserializer).map(
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
