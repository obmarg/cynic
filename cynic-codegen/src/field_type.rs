use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::{
    generic_param::{GenericConstraint, GenericParameter},
    ident::PathExt,
    schema, Ident, TypeIndex,
};

#[derive(Debug, Clone)]
pub enum FieldType {
    Scalar(Ident, bool),
    BuiltInScalar(syn::Path, bool),
    Enum(Ident, bool),
    InputObject(Ident, bool),
    Other(Ident, bool),
    List(Box<FieldType>, bool),
}

impl FieldType {
    pub fn from_schema_type(schema_type: &schema::Type, type_index: &TypeIndex) -> Self {
        FieldType::from_schema_type_internal(schema_type, type_index, true)
    }

    fn from_schema_type_internal(
        schema_type: &schema::Type,
        type_index: &TypeIndex,
        nullable: bool,
    ) -> Self {
        use schema::Type;

        match schema_type {
            Type::NonNullType(inner_type) => {
                FieldType::from_schema_type_internal(inner_type, type_index, false)
            }
            Type::ListType(inner_type) => FieldType::List(
                Box::new(FieldType::from_schema_type_internal(
                    inner_type, type_index, true,
                )),
                nullable,
            ),
            Type::NamedType(name) => {
                if type_index.is_scalar(name) {
                    FieldType::Scalar(Ident::for_type(name), nullable)
                } else if type_index.is_enum(name) {
                    FieldType::Enum(Ident::for_type(name), nullable)
                } else if type_index.is_input_object(name) {
                    FieldType::InputObject(Ident::for_type(name), nullable)
                } else if name == "Int" {
                    FieldType::BuiltInScalar(syn::parse_quote! {i32}, nullable)
                } else if name == "Float" {
                    FieldType::BuiltInScalar(syn::parse_quote! {f64}, nullable)
                } else if name == "Boolean" {
                    FieldType::BuiltInScalar(syn::parse_quote! {bool}, nullable)
                } else if name == "String" {
                    FieldType::BuiltInScalar(syn::parse_quote! {String}, nullable)
                } else if name == "ID" {
                    FieldType::BuiltInScalar(syn::parse_quote! { ::cynic::Id }, nullable)
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
            FieldType::BuiltInScalar(_, _) => true,
            _ => false,
        }
    }

    pub fn is_list(&self) -> bool {
        matches!(self, FieldType::List(_, _))
    }

    pub fn contains_enum(&self) -> bool {
        match self {
            FieldType::List(inner, _) => inner.contains_enum(),
            FieldType::Enum(_, _) => true,
            _ => false,
        }
    }

    pub fn contains_input_object(&self) -> bool {
        match self {
            FieldType::List(inner, _) => inner.contains_enum(),
            FieldType::InputObject(_, _) => true,
            _ => false,
        }
    }

    pub fn contains_leaf_value(&self) -> bool {
        match self {
            FieldType::List(inner, _) => inner.contains_scalar(),
            FieldType::Scalar(_, _) => true,
            FieldType::BuiltInScalar(_, _) => true,
            FieldType::Enum(_, _) => true,
            _ => false,
        }
    }

    /// Returns the path to the enum marker struct stored in this field, if any
    pub fn inner_enum_path(&self) -> Option<Ident> {
        match self {
            FieldType::List(inner, _) => inner.inner_enum_path(),
            FieldType::Enum(path, _) => Some(path.clone()),
            _ => None,
        }
    }

    /// Returns the path to the input object marker struct stored in this field, if any
    pub fn inner_input_object_path(&self) -> Option<Ident> {
        match self {
            FieldType::List(inner, _) => inner.inner_input_object_path(),
            FieldType::InputObject(path, _) => Some(path.clone()),
            _ => None,
        }
    }

    pub fn is_nullable(&self) -> bool {
        match self {
            FieldType::List(_, nullable) => *nullable,
            FieldType::Scalar(_, nullable) => *nullable,
            FieldType::BuiltInScalar(_, nullable) => *nullable,
            FieldType::Enum(_, nullable) => *nullable,
            FieldType::InputObject(_, nullable) => *nullable,
            FieldType::Other(_, nullable) => *nullable,
        }
    }

    pub fn as_type_lock(&self, path_to_markers: &syn::Path) -> syn::Path {
        let ident = match self {
            FieldType::List(inner, _) => return inner.as_type_lock(path_to_markers),
            FieldType::BuiltInScalar(path, _) => return path.clone(),
            FieldType::Scalar(ident, _) => ident,

            FieldType::Enum(ident, _) => ident,
            FieldType::InputObject(ident, _) => ident,
            FieldType::Other(ident, _) => ident,
        };

        let mut path = path_to_markers.clone();
        path.push(ident);
        path
    }

    /// Returns a wrapper path suitable for use in the second parameter to the `InputType` trait.
    /// e.g. `Nullable<NamedType>` for `Int`, `NamedType` for `Int!`
    pub fn wrapper_path(&self) -> Result<TokenStream, crate::Errors> {
        match self {
            FieldType::List(inner, nullable) => {
                let inner_path = inner.wrapper_path()?;
                if *nullable {
                    Ok(quote! { ::cynic::inputs::Nullable<::cynic::inputs::List<#inner_path>> })
                } else {
                    Ok(quote! { ::cynic::inputs::List<#inner_path> })
                }
            }
            FieldType::Scalar(_, nullable)
            | FieldType::BuiltInScalar(_, nullable)
            | FieldType::Enum(_, nullable)
            | FieldType::InputObject(_, nullable) => {
                if *nullable {
                    Ok(quote! { ::cynic::inputs::Nullable<::cynic::inputs::NamedType> })
                } else {
                    Ok(quote! { ::cynic::inputs::NamedType })
                }
            }
            _ => Err(syn::Error::new(
                Span::call_site(),
                "Arguments must be scalars, enums or input objects",
            )
            .into()),
        }
    }

    pub fn as_required(&self) -> Self {
        match self {
            FieldType::List(inner, _) => FieldType::List(inner.clone(), false),
            FieldType::Scalar(type_path, _) => FieldType::Scalar(type_path.clone(), false),
            FieldType::BuiltInScalar(ident, _) => FieldType::BuiltInScalar(ident.clone(), false),
            FieldType::Enum(type_path, _) => FieldType::Enum(type_path.clone(), false),
            FieldType::InputObject(type_path, _) => {
                FieldType::InputObject(type_path.clone(), false)
            }
            FieldType::Other(type_path, _) => FieldType::Other(type_path.clone(), false),
        }
    }

    /// Generates a call to selection set functions for this type.
    ///
    /// Where inner_select is a call to the sub-fields to select (or the scalar
    /// function if that's necceasry here)
    pub fn selection_set_call(&self, inner_select: TokenStream) -> TokenStream {
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
            FieldType::InputObject(_, _) => {
                panic!("Input objects should never be selected, what's going on here...")
            }
            FieldType::Enum(_, _)
            | FieldType::Other(_, _)
            | FieldType::Scalar(_, _)
            | FieldType::BuiltInScalar(_, _) => {
                quote! { #inner_select }
            }
        }
    }

    /// Creates the output DecodesTo for a selector function that represents
    /// this type.  For example if inner is `T` and this is an optional
    /// vec this will spit out Option<Vec<T>>
    pub fn decodes_to(&self, inner_token: TokenStream) -> TokenStream {
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
            FieldType::InputObject(_, _) => {
                panic!("Input objects should never be selected, what's going on here...")
            }
            FieldType::Enum(_, _)
            | FieldType::Other(_, _)
            | FieldType::Scalar(_, _)
            | FieldType::BuiltInScalar(_, _) => {
                quote! { #inner_token }
            }
        }
    }

    // Converts a FieldType to a rust type definition.
    //
    // generic_inner_type should be provided if the inner type doesn't represent a
    // concrete type and needs to use a type parameter defined at an outer level.
    // The name of the type parameter should be passed in to generic_inner_type.
    pub fn to_tokens(
        &self,
        generic_inner_type: Option<Ident>,
        path_to_types: syn::Path,
    ) -> TokenStream {
        let nullable = self.is_nullable();
        let rust_type = match (self, &generic_inner_type) {
            (FieldType::List(inner_type, _), _) => {
                let inner_type = inner_type.to_tokens(generic_inner_type, path_to_types);
                quote! { Vec<#inner_type> }
            }
            (_, Some(generic_type)) => {
                quote! { #generic_type }
            }
            (FieldType::BuiltInScalar(path, _), _) => {
                quote! { #path }
            }
            (FieldType::Scalar(ident, _), _) => {
                quote! { #path_to_types::#ident }
            }
            (FieldType::Other(typename, _), _) => {
                quote! { #path_to_types::#typename }
            }
            (FieldType::Enum(name, _), _) => {
                quote! { #path_to_types::#name }
            }
            (FieldType::InputObject(name, _), _) => {
                quote! { #path_to_types::#name }
            }
        };

        if nullable {
            quote! { Option<#rust_type> }
        } else {
            rust_type
        }
    }

    /// Returns a GenericParameter with the given name if applicable.
    ///
    /// Generic parameters are required when working with enum or input object
    /// fields, as we can't name their concrete type just constrain functions
    /// by the Enum or InputObject trait.
    pub fn generic_parameter(&self, name: Ident) -> Option<GenericParameter> {
        if let Some(path) = self.inner_enum_path() {
            Some(GenericParameter {
                name,
                constraint: GenericConstraint::Enum(path),
            })
        } else {
            self.inner_input_object_path().map(|path| GenericParameter {
                name,
                constraint: GenericConstraint::InputObject(path),
            })
        }
    }
}
