//! Utilities for parsing attributes
use std::collections::HashMap;
use std::{fmt::Display, str::FromStr};

use proc_macro2::Span;
use quote::format_ident;

#[derive(Debug, Clone)]
pub struct Attribute {
    pub value: String,
    pub span: Span,
}

impl From<syn::LitStr> for Attribute {
    fn from(s: syn::LitStr) -> Self {
        Attribute {
            value: s.value(),
            span: s.span(),
        }
    }
}

/// Extracts key value pairs from a `#[cynic(key=value)]` attribute.
pub fn extract_meta_attrs<AttributeKey>(
    attrs: &Vec<syn::Attribute>,
) -> Result<(HashMap<AttributeKey, Attribute>, Span), syn::Error>
where
    AttributeKey: FromStr + Eq + std::hash::Hash,
    <AttributeKey as FromStr>::Err: Display,
{
    use syn::{spanned::Spanned, Lit, Meta, MetaList, MetaNameValue, NestedMeta};

    let cynic_meta = attrs
        .iter()
        .find(|a| a.path.is_ident(&format_ident!("cynic")))
        .ok_or(syn::Error::new(
            Span::call_site(),
            "cynic attribute not provided",
        ))
        .and_then(|attr| attr.parse_meta())?;

    let mut attr_map = HashMap::new();

    if let Meta::List(MetaList { nested, .. }) = &cynic_meta {
        for meta in nested {
            if let NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. })) = meta {
                if let Some(ident) = path.get_ident() {
                    let attr_name = ident
                        .to_string()
                        .parse()
                        .map_err(|e| syn::Error::new(ident.span(), &e))?;

                    if let Lit::Str(lit_str) = lit {
                        attr_map.insert(attr_name, lit_str.clone().into());
                    } else {
                        // TODO: Re-factor this into something nicer...
                        // Could probably return an Error enum and move the strings
                        // elsewhere.
                        // Could potentially also do this with combinators or similar..
                        return Err(syn::Error::new(
                            lit.span(),
                            "values in the cynic attribute should be string literals",
                        ));
                    }
                } else {
                    return Err(syn::Error::new(
                        path.span(),
                        "keys in the cynic attribute should be a single identifier",
                    ));
                }
            } else {
                return Err(syn::Error::new(
                    meta.span(),
                    "The cynic attribute accepts a list of key=\"value\" pairs",
                ));
            }
        }
    } else {
        return Err(syn::Error::new(
            cynic_meta.span(),
            "The cynic attribute accepts a list of key=\"value\" pairs",
        ));
    }

    return Ok((attr_map, cynic_meta.span()));
}
