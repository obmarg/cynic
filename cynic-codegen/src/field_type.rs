use graphql_parser::schema;
use proc_macro2::TokenStream;
use std::collections::HashSet;

use crate::{Ident, TypePath};

#[derive(Debug, Clone)]
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

    pub fn as_type_lock(&self) -> TypePath {
        match self {
            FieldType::List(inner, _) => inner.as_type_lock(),
            // TODO: I think this is wrong for scalars, but whatever.
            FieldType::Scalar(type_path, _) => type_path.clone(),
            FieldType::Other(type_path, _) => type_path.clone(),
        }
    }

    pub fn as_required(&self) -> Self {
        match self {
            FieldType::List(inner, _) => FieldType::List(inner.clone(), false),
            FieldType::Scalar(type_path, _) => FieldType::Scalar(type_path.clone(), false),
            FieldType::Other(type_path, _) => FieldType::Other(type_path.clone(), false),
        }
    }

    /// Extracts the inner type from a syn::Type that corresponds with this type.
    ///
    /// This takes some (potentially nested) Options & Vectors and extracts enough layers
    /// to get at the inner type.
    ///
    /// For example if this is a `[Int]` and we passed in a `Option<Vec<Option<u8>>>`
    /// this would return the u8.
    ///
    /// This is useful when calling QueryFragment::selection_set functions in
    /// our derived QueryFragment as only the inner type implements QueryFragment
    /// and we use selection set functions manully to build up the required &
    /// list types.
    pub fn get_inner_type_from_syn(&self, ty: &syn::Type) -> syn::Type {
        use syn::{GenericArgument, PathArguments, Type, TypePath};
        if self.is_nullable() {
            // Strip off a top level nullable & recurse...
            if let Type::Path(expr) = ty {
                if let Some(segment) = expr.path.segments.first() {
                    if segment.ident.to_string() == "Option" {
                        if let PathArguments::AngleBracketed(expr) = &segment.arguments {
                            if let Some(GenericArgument::Type(ty)) = expr.args.first() {
                                return self.as_required().get_inner_type_from_syn(ty);
                            }
                        }
                    }
                }
            }
        } else if let FieldType::List(inner_self, _) = self {
            if let Type::Path(expr) = ty {
                if let Some(segment) = expr.path.segments.first() {
                    if segment.ident.to_string() == "Vec" {
                        if let PathArguments::AngleBracketed(expr) = &segment.arguments {
                            if let Some(GenericArgument::Type(ty)) = expr.args.first() {
                                return inner_self.get_inner_type_from_syn(ty);
                            }
                        }
                    }
                }
            }
        }
        ty.clone()
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
