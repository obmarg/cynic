use crate::casings::CasingExt;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

use crate::query_parsing::Variable;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArgumentStruct<'query, 'schema> {
    pub name: String,
    pub fields: Vec<ArgumentStructField<'query, 'schema>>,
}

impl<'query, 'schema> ArgumentStruct<'query, 'schema> {
    pub fn new(name: String, fields: Vec<ArgumentStructField<'query, 'schema>>) -> Self {
        ArgumentStruct { name, fields }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ArgumentStructField<'query, 'schema> {
    Variable(Variable<'query, 'schema>),
    NestedStruct(String),
}

impl<'query, 'schema> ArgumentStructField<'query, 'schema> {
    pub fn name(&self) -> String {
        match self {
            ArgumentStructField::Variable(var) => var.name.0.to_string().to_snake_case(),
            ArgumentStructField::NestedStruct(type_name) => type_name.to_snake_case(),
        }
    }

    pub fn type_spec(&self) -> String {
        match self {
            ArgumentStructField::Variable(var) => var.value_type.type_spec().to_string(),
            ArgumentStructField::NestedStruct(type_name) => type_name.clone(),
        }
    }
}

impl std::fmt::Display for ArgumentStruct<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use super::indented;
        use std::fmt::Write;

        writeln!(f, "#[derive(cynic::FragmentArguments, Debug)]")?;
        writeln!(f, "pub struct {} {{", self.name)?;

        for field in &self.fields {
            write!(indented(f, 4), "{}", field)?;
        }
        writeln!(f, "}}")
    }
}

impl ToTokens for ArgumentStruct<'_, '_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = Ident::new(&self.name, Span::call_site());
        let fields = &self.fields;

        tokens.extend(quote! {
            #[derive(cynic::FragmentArguments, Debug)]
            pub struct #name {
                #(#fields),*
            }
        })
    }
}

impl std::fmt::Display for ArgumentStructField<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", super::Field::new(&self.name(), &self.type_spec()))
    }
}

impl ToTokens for ArgumentStructField<'_, '_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = Ident::new(&self.name(), Span::call_site());
        let typ = self
            .type_spec()
            .parse::<proc_macro2::TokenStream>()
            .unwrap();

        tokens.extend(quote! {
            pub #name: #typ
        })
    }
}
