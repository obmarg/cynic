use graphql_parser::schema;
use proc_macro2::TokenStream;
use std::collections::HashSet;

use crate::ident::Ident;

#[derive(Debug)]
pub enum FieldType {
    Scalar(Ident, bool),
    Other(Ident, bool),
    List(Box<FieldType>, bool),
}

impl FieldType {
    pub fn from_schema_type(schema_type: &schema::Type, scalar_names: &HashSet<String>) -> Self {
        FieldType::from_schema_type_internal(schema_type, scalar_names, true)
    }

    fn from_schema_type_internal(
        schema_type: &schema::Type,
        scalar_names: &HashSet<String>,
        nullable: bool,
    ) -> Self {
        use schema::Type;

        match schema_type {
            Type::NonNullType(inner_type) => {
                FieldType::from_schema_type_internal(inner_type, scalar_names, false)
            }
            Type::ListType(inner_type) => FieldType::List(
                Box::new(FieldType::from_schema_type_internal(
                    inner_type,
                    scalar_names,
                    true,
                )),
                nullable,
            ),
            Type::NamedType(name) => {
                if scalar_names.contains(name) {
                    FieldType::Scalar(Ident::for_inbuilt_scalar(name), nullable)
                } else if name == "Int" {
                    FieldType::Scalar(Ident::for_inbuilt_scalar("i64"), nullable)
                } else if name == "Float" {
                    FieldType::Scalar(Ident::for_inbuilt_scalar("f64"), nullable)
                } else if name == "Boolean" {
                    FieldType::Scalar(Ident::for_inbuilt_scalar("bool"), nullable)
                } else if name == "JSON" {
                    // TODO: figure out how to use ident like this...
                    FieldType::Scalar(Ident::for_inbuilt_scalar("serde_json::Value"), nullable)
                } else if name == "String" || name == "ID" {
                    // TODO: Could do something more sensible for IDs here...
                    FieldType::Scalar(Ident::for_inbuilt_scalar("String"), nullable)
                } else {
                    FieldType::Other(Ident::for_type(name), nullable)
                }
            }
        }
    }

    pub fn contains_scalar(&self) -> bool {
        match self {
            FieldType::List(inner, _) => inner.contains_scalar(),
            FieldType::Scalar(_, _) => true,
            FieldType::Other(_, _) => false,
        }
    }

    pub fn is_nullable(&self) -> bool {
        match self {
            FieldType::List(_, nullable) => nullable.clone(),
            FieldType::Scalar(_, nullable) => nullable.clone(),
            FieldType::Other(_, nullable) => nullable.clone(),
        }
    }
}

impl quote::ToTokens for FieldType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let nullable = self.is_nullable();
        let rust_type = match self {
            FieldType::List(inner_type, _) => quote! { Vec<#inner_type> },
            FieldType::Scalar(typename, _) => quote! { #typename },
            FieldType::Other(typename, _) => quote! { #typename },
        };

        if nullable {
            tokens.append_all(quote! { Option<#rust_type> });
        } else {
            tokens.append_all(rust_type);
        }
    }
}
