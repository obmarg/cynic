use std::borrow::Borrow;

use quote::format_ident;

use crate::schema::types::*;

use crate::idents::to_snake_case;

use super::keywords::transform_keywords;

/// Ident for a type
#[derive(Clone, Copy, Debug)]
pub struct TypeMarkerIdent<'a> {
    graphql_name: &'a str,
}

/// Ident for a field of a type
#[derive(Clone, Copy, Debug)]
pub struct FieldMarkerIdent<'a> {
    graphql_name: &'a str,
}

/// A module that contains everything associated with a field.
#[derive(Clone, Copy, Debug)]
pub struct FieldMarkerModule<'a> {
    type_name: &'a str,
}

/// A module that contains everything associated with an argument to a field
#[derive(Clone, Copy, Debug)]
pub struct ArgumentMarkerModule<'a> {
    type_name: &'a str,
    field_name: &'a str,
}

/// Marker to the type of a field - handles options & vecs and whatever the inner
/// type is
#[derive(Clone)]
pub struct TypeRefMarker<'a, T> {
    type_ref: &'a TypeRef<'a, T>,
}

impl<T> TypeRefMarker<'_, T> {
    pub fn to_path(&self, path_to_markers: &syn::Path) -> syn::Path {
        use syn::parse_quote;

        match &self.type_ref {
            TypeRef::Named(name, _, _) => {
                TypeMarkerIdent { graphql_name: name }.to_path(path_to_markers)
            }
            TypeRef::List(inner) => {
                let inner_path = TypeRefMarker {
                    type_ref: inner.as_ref(),
                }
                .to_path(path_to_markers);

                parse_quote! {
                    Vec<#inner_path>
                }
            }
            TypeRef::Nullable(inner) => {
                let inner_path = TypeRefMarker {
                    type_ref: inner.as_ref(),
                }
                .to_path(path_to_markers);

                parse_quote! {
                    Option<#inner_path>
                }
            }
        }
    }
}

impl<'a> TypeMarkerIdent<'a> {
    pub fn with_graphql_name(graphql_name: &'a str) -> Self {
        TypeMarkerIdent { graphql_name }
    }

    pub fn to_path(self, path_to_markers: &syn::Path) -> syn::Path {
        let mut path = path_to_markers.clone();
        path.push(proc_macro2::Ident::from(self));
        path
    }
}

impl From<TypeMarkerIdent<'_>> for proc_macro2::Ident {
    fn from(val: TypeMarkerIdent<'_>) -> Self {
        format_ident!("{}", transform_keywords(val.graphql_name))
    }
}

impl<'a> FieldMarkerModule<'a> {
    pub fn ident(&self) -> proc_macro2::Ident {
        format_ident!("{}_fields", to_snake_case(self.type_name))
    }

    pub fn to_path(self, path_to_markers: &syn::Path) -> syn::Path {
        let mut path = path_to_markers.clone();
        path.push(self.ident());
        path
    }
}

impl<'a> FieldMarkerIdent<'a> {
    pub fn to_path(self, path_to_markers: &syn::Path) -> syn::Path {
        let mut path = path_to_markers.clone();
        path.push(proc_macro2::Ident::from(self));
        path
    }
}

impl From<FieldMarkerIdent<'_>> for proc_macro2::Ident {
    fn from(val: FieldMarkerIdent<'_>) -> Self {
        if val.graphql_name == "_" {
            return format_ident!("_Underscore");
        }

        format_ident!("{}", transform_keywords(val.graphql_name))
    }
}

impl ArgumentMarkerModule<'_> {
    pub fn ident(&self) -> proc_macro2::Ident {
        format_ident!("{}_arguments", to_snake_case(self.field_name))
    }

    pub fn to_path(self, path_to_markers: &syn::Path) -> syn::Path {
        let mut path = path_to_markers.clone();
        path.push(
            FieldMarkerModule {
                type_name: self.type_name,
            }
            .ident(),
        );
        path.push(self.ident());
        path
    }
}

macro_rules! marker_ident_for_named {
    () => {};
    ($kind:ident) => {
        impl<'a> $kind<'a> {
            pub fn marker_ident(&self) -> TypeMarkerIdent<'a> {
                TypeMarkerIdent {
                    graphql_name: self.name
                }
            }
        }
    };
    ($kind:ident, $($rest:ident),+) => {
        marker_ident_for_named!($kind);
        marker_ident_for_named!($($rest),*);
    };
}

marker_ident_for_named!(
    ObjectType,
    EnumType,
    InterfaceType,
    UnionType,
    InputObjectType,
    ScalarType
);

impl<'a> Field<'a> {
    pub fn marker_ident(&self) -> FieldMarkerIdent<'a> {
        FieldMarkerIdent {
            graphql_name: self.name.as_str(),
        }
    }
}

impl<'a> InputValue<'a> {
    pub fn marker_ident(&self) -> FieldMarkerIdent<'a> {
        FieldMarkerIdent {
            graphql_name: self.name.as_str(),
        }
    }
}

impl<'a> ObjectType<'a> {
    pub fn field_module(&self) -> FieldMarkerModule<'a> {
        FieldMarkerModule {
            type_name: self.name,
        }
    }
}

impl<'a> InterfaceType<'a> {
    pub fn field_module(&self) -> FieldMarkerModule<'a> {
        FieldMarkerModule {
            type_name: self.name,
        }
    }
}

impl<'a> InputObjectType<'a> {
    pub fn field_module(&self) -> FieldMarkerModule<'a> {
        FieldMarkerModule {
            type_name: self.name,
        }
    }
}

impl<'a> Field<'a> {
    pub fn argument_module(&self) -> ArgumentMarkerModule<'a> {
        ArgumentMarkerModule {
            type_name: self.parent_type_name,
            field_name: self.name.graphql_name,
        }
    }
}

impl<'a> ObjectRef<'a> {
    pub fn marker_ident(&self) -> TypeMarkerIdent<'a> {
        TypeMarkerIdent {
            graphql_name: self.0,
        }
    }
}

impl<'a> InterfaceRef<'a> {
    pub fn marker_ident(&self) -> TypeMarkerIdent<'a> {
        TypeMarkerIdent {
            graphql_name: self.0,
        }
    }
}

impl<'a, T> TypeRef<'a, T> {
    pub fn marker_type(&'a self) -> TypeRefMarker<'a, T> {
        TypeRefMarker { type_ref: self }
    }
}

trait PathExt {
    fn push(&mut self, ident: impl Borrow<proc_macro2::Ident>);
}

impl PathExt for syn::Path {
    fn push(&mut self, ident: impl Borrow<proc_macro2::Ident>) {
        self.segments.push(ident.borrow().clone().into())
    }
}
