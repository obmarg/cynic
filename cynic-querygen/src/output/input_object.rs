use std::fmt::Write;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

use super::indented;
use crate::casings::CasingExt;
use crate::schema::InputField;

#[derive(Debug, PartialEq)]
pub struct InputObject<'schema> {
    pub name: String,
    pub fields: Vec<InputField<'schema>>,
}

impl std::fmt::Display for InputObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#[derive(cynic::InputObject, Debug)]")?;
        if self.name != self.name.to_pascal_case() {
            writeln!(f, "#[cynic(graphql_type = \"{}\")]", self.name)?;
        }
        writeln!(f, "pub struct {} {{", self.name.to_pascal_case())?;

        for field in &self.fields {
            let mut f = indented(f, 4);

            let name = field.name.0.to_snake_case();
            let type_spec = field.type_spec();
            let mut output = super::Field::new(&name, &type_spec);

            if name.to_camel_case() != field.name.0 {
                // If a snake -> camel casing roundtrip is not lossless
                // we need to explicitly rename this field
                output.add_rename(field.name.0);
            }

            write!(f, "{}", output)?;
        }

        writeln!(f, "}}")
    }
}

impl ToTokens for InputObject<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let rename = if self.name != self.name.to_pascal_case() {
            let name = &self.name;
            Some(quote! {
                #[cynic(graphql_type = #name)]
            })
        } else {
            None
        };
        let pascal_name = Ident::new(&self.name.to_pascal_case(), Span::call_site());

        let fields = self.fields.iter().map(|field| {
            let rename = if field.name.0.to_snake_case().to_camel_case() != field.name.0 {
                // If a snake -> camel casing roundtrip is not lossless
                // we need to explicitly rename this field
                let rename = &field.name.0;

                Some(quote! {
                    #[cynic(rename = #rename)]
                })
            } else {
                None
            };

            let field_name = Ident::new(&field.name.0.to_snake_case(), Span::call_site());
            let field_type = Ident::new(&field.type_spec(), Span::call_site());

            quote! {
                #rename
                pub #field_name : #field_type
            }
        });

        tokens.extend(quote! {
            #[derive(cynic::InputObject, Debug)]
            #rename
            pub struct #pascal_name {
                #(#fields),*
            }
        });
    }
}
