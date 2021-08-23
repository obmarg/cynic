use inflector::Inflector;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::fmt::Write;

use super::indented;
use crate::schema::EnumDetails;

impl std::fmt::Display for EnumDetails<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = self.name;

        writeln!(f, "#[derive(cynic::Enum, Clone, Copy, Debug)]")?;
        if type_name != type_name.to_pascal_case() {
            writeln!(f, "#[cynic(graphql_type = \"{}\")]", type_name)?;
        }
        writeln!(f, "pub enum {} {{", type_name.to_pascal_case())?;

        for variant in &self.values {
            let mut f = indented(f, 4);

            if variant.to_pascal_case().to_screaming_snake_case() != *variant {
                // If a pascal -> screaming snake casing roundtrip is not lossless
                // we need to explicitly rename this variant
                writeln!(f, "#[cynic(rename = \"{}\")]", variant)?;
            }

            writeln!(f, "{},", variant.to_pascal_case())?;
        }
        writeln!(f, "}}")
    }
}

impl ToTokens for EnumDetails<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let type_name = &self.name.to_pascal_case();

        let rename = if self.name != type_name {
            Some(quote! {
                #[cynic(graphql_type = #type_name)]
            })
        } else {
            None
        };

        let values = self.values.iter().map(|variant| {
            let renamed = &variant.to_pascal_case().to_screaming_snake_case();
            let variant_name = Ident::new(&renamed, Span::call_site());
            let variant = Ident::new(&variant, Span::call_site());
            let rename = if variant != &renamed {
                Some(quote! {
                    #[cynic(rename = #variant_name)]
                })
            } else {
                None
            };
            quote! {
                #rename
                #variant
            }
        });

        let type_name = Ident::new(&type_name, Span::call_site());
        tokens.extend(quote! {
            #[derive(cynic::Enum, Clone, Copy, Debug)]
            #rename

            pub enum #type_name {
                #(#values),*
            }
        });
    }
}
