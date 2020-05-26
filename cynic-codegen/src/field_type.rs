use graphql_parser::schema;
use proc_macro2::TokenStream;

use crate::{Ident, TypeIndex, TypePath};

#[derive(Debug, Clone)]
pub enum FieldType {
    Scalar(Ident, bool),
    Enum(Ident, bool),
    Other(Ident, bool),
    List(Box<FieldType>, bool),
}

impl FieldType {
    pub fn from_schema_type(
        schema_type: &schema::Type,
        type_path: TypePath,
        type_index: &TypeIndex,
    ) -> Self {
        FieldType::from_schema_type_internal(schema_type, type_path, type_index, true)
    }

    fn from_schema_type_internal(
        schema_type: &schema::Type,
        type_path: TypePath,
        type_index: &TypeIndex,
        nullable: bool,
    ) -> Self {
        use schema::Type;

        match schema_type {
            Type::NonNullType(inner_type) => {
                FieldType::from_schema_type_internal(inner_type, type_path, type_index, false)
            }
            Type::ListType(inner_type) => FieldType::List(
                Box::new(FieldType::from_schema_type_internal(
                    inner_type, type_path, type_index, true,
                )),
                nullable,
            ),
            Type::NamedType(name) => {
                if type_index.is_scalar(name) {
                    FieldType::Scalar(Ident::for_type(name), nullable)
                } else if type_index.is_enum(name) {
                    FieldType::Enum(Ident::for_type(name), nullable)
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
                    FieldType::Other(Ident::for_type(name).into(), nullable)
                }
            }
        }
    }

    pub fn contains_scalar(&self) -> bool {
        match self {
            FieldType::List(inner, _) => inner.contains_scalar(),
            FieldType::Scalar(_, _) => true,
            FieldType::Enum(_, _) => false,
            FieldType::Other(_, _) => false,
        }
    }

    pub fn contains_enum(&self) -> bool {
        match self {
            FieldType::List(inner, _) => inner.contains_scalar(),
            FieldType::Scalar(_, _) => false,
            FieldType::Enum(_, _) => true,
            FieldType::Other(_, _) => false,
        }
    }

    /// Returns the path to the enum marker struct stored in this field, if any
    pub fn inner_enum_path(&self) -> Option<Ident> {
        match self {
            FieldType::List(inner, _) => inner.inner_enum_path(),
            FieldType::Scalar(_, _) => None,
            FieldType::Enum(path, _) => Some(path.clone()),
            FieldType::Other(_, _) => None,
        }
    }

    pub fn is_nullable(&self) -> bool {
        match self {
            FieldType::List(_, nullable) => nullable.clone(),
            FieldType::Scalar(_, nullable) => nullable.clone(),
            FieldType::Enum(_, nullable) => nullable.clone(),
            FieldType::Other(_, nullable) => nullable.clone(),
        }
    }

    pub fn as_type_lock(&self, path_to_types: TypePath) -> TypePath {
        match self {
            FieldType::List(inner, _) => inner.as_type_lock(path_to_types),
            // TODO: I think this is wrong for scalars, but whatever.
            FieldType::Scalar(ident, _) => TypePath::concat(&[path_to_types, ident.clone().into()]),
            FieldType::Enum(_, _) => TypePath::void(),
            FieldType::Other(ident, _) => TypePath::concat(&[path_to_types, ident.clone().into()]),
        }
    }

    pub fn as_required(&self) -> Self {
        match self {
            FieldType::List(inner, _) => FieldType::List(inner.clone(), false),
            FieldType::Scalar(type_path, _) => FieldType::Scalar(type_path.clone(), false),
            FieldType::Enum(type_path, _) => FieldType::Enum(type_path.clone(), false),
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
        use syn::{GenericArgument, PathArguments, Type};
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

            // Note: as we no longer require Opt<Vec<Opt<>> to match our types precisely
            // we go ahead and recurse anyway here...
            return self.as_required().get_inner_type_from_syn(ty);
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

            // Note: as we no longer require rust types to match GQL types precisely
            // we go ahead and recurse anyway here...
            return inner_self.get_inner_type_from_syn(ty);
        }
        ty.clone()
    }

    /// Generates a call to selection set functions for this type.
    ///
    /// Where inner_select is a call to the sub-fields to select (or the scalar
    /// function if that's necceasry here)
    pub fn selection_set_call(&self, inner_select: TokenStream) -> TokenStream {
        use quote::quote;

        if self.is_nullable() {
            let inner = self.as_required().selection_set_call(inner_select);
            return quote! {
                ::cynic::selection_set::option(#inner)
            };
        }

        match self {
            FieldType::List(inner_type, _) => {
                let inner = inner_type.selection_set_call(inner_select);
                quote! {
                    ::cynic::selection_set::vec(#inner)
                }
            }
            FieldType::Enum(_, _) | FieldType::Other(_, _) | FieldType::Scalar(_, _) => {
                quote! { #inner_select }
            }
        }
    }

    /// Creates the output DecodesTo for a selector function that represents
    /// this type.  For example if inner is `T` and this is an optional
    /// vec this will spit out Option<Vec<T>>
    pub fn decodes_to(&self, inner_token: TokenStream) -> TokenStream {
        // TODO: Probably possible to combine this with the ToTokens implementation below.
        use quote::quote;

        if self.is_nullable() {
            let inner = self.as_required().decodes_to(inner_token);
            return quote! {
                Option<#inner>
            };
        }

        match self {
            FieldType::List(inner_type, _) => {
                let inner = inner_type.decodes_to(inner_token);
                quote! {
                    Vec<#inner>
                }
            }
            FieldType::Enum(_, _) | FieldType::Other(_, _) | FieldType::Scalar(_, _) => {
                quote! { #inner_token }
            }
        }
    }

    // Converts a FieldType to a rust type definition.
    //
    // generic_inner_type should be provided if the inner type doesn't represent a
    // concrete type and needs to use a type parameter defined at an outer level.
    // The name of the type parameter should be passed in to generic_inner_type.
    pub fn to_tokens(&self, generic_inner_type: Option<Ident>) -> TokenStream {
        use quote::quote;

        let nullable = self.is_nullable();
        let rust_type = match (self, generic_inner_type) {
            (FieldType::List(_, _), Some(generic)) => quote! { Vec<#generic> },
            (FieldType::List(inner_type, _), _) => quote! { Vec<#inner_type> },
            (_, Some(generic)) => quote! { #generic },
            (FieldType::Scalar(typename, _), _) => quote! { #typename },
            (FieldType::Other(typename, _), _) => quote! { #typename },
            (FieldType::Enum(typename, _), _) => quote! { #typename },
        };

        if nullable {
            quote! { Option<#rust_type> }
        } else {
            rust_type
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
            FieldType::Enum(typename, _) => quote! { #typename },
        };

        if nullable {
            tokens.append_all(quote! { Option<#rust_type> });
        } else {
            tokens.append_all(rust_type);
        }
    }
}
