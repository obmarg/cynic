use graphql_parser::schema;
use proc_macro2::TokenStream;
use std::collections::HashSet;

use super::type_path::TypePath;
use crate::ident::Ident;

#[derive(Debug)]
pub enum FieldType {
    Scalar(TypePath, bool),
    Other(TypePath, bool),
    List(Box<FieldType>, bool),
}

impl FieldType {
    pub fn from_schema_type(
        schema_type: &schema::Type,
        type_path: TypePath,
        scalar_names: &HashSet<String>,
    ) -> Self {
        FieldType::from_schema_type_internal(schema_type, type_path, scalar_names, true)
    }

    fn from_schema_type_internal(
        schema_type: &schema::Type,
        type_path: TypePath,
        scalar_names: &HashSet<String>,
        nullable: bool,
    ) -> Self {
        use schema::Type;

        match schema_type {
            Type::NonNullType(inner_type) => {
                FieldType::from_schema_type_internal(inner_type, type_path, scalar_names, false)
            }
            Type::ListType(inner_type) => FieldType::List(
                Box::new(FieldType::from_schema_type_internal(
                    inner_type,
                    type_path,
                    scalar_names,
                    true,
                )),
                nullable,
            ),
            Type::NamedType(name) => {
                if scalar_names.contains(name) {
                    FieldType::Scalar(
                        TypePath::concat(&[type_path, Ident::for_inbuilt_scalar(name).into()]),
                        nullable,
                    )
                } else if name == "Int" {
                    FieldType::Scalar(Ident::for_inbuilt_scalar("i64").into(), nullable)
                } else if name == "Float" {
                    FieldType::Scalar(Ident::for_inbuilt_scalar("f64").into(), nullable)
                } else if name == "Boolean" {
                    FieldType::Scalar(Ident::for_inbuilt_scalar("bool").into(), nullable)
                } else if name == "String" || name == "ID" {
                    // TODO: Could do something more sensible for IDs here...
                    FieldType::Scalar(Ident::for_inbuilt_scalar("String").into(), nullable)
                } else {
                    FieldType::Other(
                        TypePath::concat(&[type_path, Ident::for_type(name).into()]),
                        nullable,
                    )
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
