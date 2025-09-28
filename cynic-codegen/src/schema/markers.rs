use std::borrow::{Borrow, Cow};

use proc_macro2::Span;
use quote::format_ident;
use syn::spanned::Spanned;
use syn::{GenericArgument, PathArguments, PathSegment, Token};

use crate::schema::types::*;

use crate::idents::to_snake_case;

use super::keywords::transform_keywords;

/// Ident for a type
#[derive(Clone, Debug)]
pub struct TypeMarkerIdent<'a> {
    graphql_name: Cow<'a, str>,
}

/// Ident for a field of a type
#[derive(Clone, Debug)]
pub struct FieldMarkerIdent<'a> {
    graphql_name: &'a str,
}

/// A module that contains everything associated with a field.
#[derive(Clone, Debug)]
pub struct FieldMarkerModule<'a> {
    type_name: Cow<'a, str>,
}

/// A module that contains everything associated with an argument to a field
#[derive(Debug)]
pub struct ArgumentMarkerModule<'a> {
    type_name: Cow<'a, str>,
    field_name: Cow<'a, str>,
}

/// Ident for a directive marker type
#[derive(Clone, Debug)]
pub struct DirectiveMarkerIdent<'a> {
    graphql_name: &'a str,
}

/// A module that contains everything associated with an argument to a directive
#[derive(Debug)]
pub struct DirectiveArgumentMarkerModule<'a> {
    directive_name: &'a str,
}

/// Marker to the type of a field - handles options & vecs and whatever the inner
/// type is
#[derive(Clone)]
pub struct TypeRefMarker<'a, T> {
    type_ref: &'a TypeRef<'a, T>,
}

impl<T> TypeRefMarker<'_, T> {
    pub fn to_path(&self, schema_module_path: &syn::Path) -> syn::Path {
        match &self.type_ref {
            TypeRef::Named(name, _) => TypeMarkerIdent {
                graphql_name: name.clone(),
            }
            .to_path(schema_module_path),
            TypeRef::List(inner) => {
                let inner_path = TypeRefMarker {
                    type_ref: inner.as_ref(),
                }
                .to_path(schema_module_path);

                // TODO: probably want a better span on these
                let span = Span::call_site();

                let mut segment = PathSegment::from(syn::Ident::new("Vec", Span::call_site()));
                segment.arguments =
                    PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                        colon2_token: None,
                        lt_token: Token![<](span),
                        args: [GenericArgument::Type(syn::Type::Path(syn::TypePath {
                            qself: None,
                            path: inner_path,
                        }))]
                        .into_iter()
                        .collect(),
                        gt_token: Token![>](span),
                    });

                segment.into()
            }
            TypeRef::Nullable(inner) => {
                let inner_path = TypeRefMarker {
                    type_ref: inner.as_ref(),
                }
                .to_path(schema_module_path);

                // TODO: probably want a better span on these
                let span = Span::call_site();

                let mut segment = PathSegment::from(syn::Ident::new("Option", Span::call_site()));
                segment.arguments =
                    PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                        colon2_token: None,
                        lt_token: Token![<](span),
                        args: [GenericArgument::Type(syn::Type::Path(syn::TypePath {
                            qself: None,
                            path: inner_path,
                        }))]
                        .into_iter()
                        .collect(),
                        gt_token: Token![>](span),
                    });

                segment.into()
            }
        }
    }
}

impl<'a> TypeMarkerIdent<'a> {
    pub fn with_graphql_name(graphql_name: &'a str) -> Self {
        TypeMarkerIdent {
            graphql_name: Cow::Borrowed(graphql_name),
        }
    }

    pub fn to_path(&self, schema_module_path: &syn::Path) -> syn::Path {
        let mut path = schema_module_path.clone();
        path.push(self.to_rust_ident());
        path
    }

    pub fn to_rust_ident(&self) -> proc_macro2::Ident {
        format_ident!("{}", transform_keywords(self.graphql_name.as_ref()))
    }
}

impl FieldMarkerModule<'_> {
    pub fn ident(&self) -> proc_macro2::Ident {
        format_ident!("{}", transform_keywords(self.type_name.as_ref()))
    }

    pub fn to_path(&self, schema_module_path: &syn::Path) -> syn::Path {
        let mut path = schema_module_path.clone();
        path.push(proc_macro2::Ident::new(
            "__fields",
            schema_module_path.span(),
        ));
        path.push(self.ident());
        path
    }
}

impl FieldMarkerIdent<'_> {
    pub fn to_path(&self, schema_module_path: &syn::Path) -> syn::Path {
        let mut path = schema_module_path.clone();
        path.push(self.to_rust_ident());
        path
    }

    pub fn to_rust_ident(&self) -> proc_macro2::Ident {
        if self.graphql_name == "_" {
            return format_ident!("_Underscore");
        }

        format_ident!("{}", transform_keywords(self.graphql_name))
    }
}

impl ArgumentMarkerModule<'_> {
    pub fn ident(&self) -> proc_macro2::Ident {
        format_ident!("_{}_arguments", to_snake_case(self.field_name.as_ref()))
    }

    pub fn to_path(&self, schema_module_path: &syn::Path) -> syn::Path {
        let mut path = FieldMarkerModule {
            type_name: self.type_name.clone(),
        }
        .to_path(schema_module_path);
        path.push(self.ident());
        path
    }
}

impl DirectiveMarkerIdent<'_> {
    pub fn to_path(&self, schema_module_path: &syn::Path) -> syn::Path {
        let mut path = schema_module_path.clone();
        path.push(self.to_rust_ident());
        path
    }

    pub fn to_rust_ident(&self) -> proc_macro2::Ident {
        format_ident!("{}", transform_keywords(self.graphql_name))
    }
}

impl DirectiveArgumentMarkerModule<'_> {
    pub fn ident(&self) -> proc_macro2::Ident {
        format_ident!("_{}_arguments", self.directive_name)
    }

    pub fn to_path(&self, schema_module_path: &syn::Path) -> syn::Path {
        let mut path = schema_module_path.clone();
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
                    graphql_name: self.name.clone()
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
    pub fn marker_ident(&'a self) -> FieldMarkerIdent<'a> {
        FieldMarkerIdent {
            graphql_name: self.name.as_str(),
        }
    }
}

impl<'a> InputValue<'a> {
    pub fn marker_ident(&'a self) -> FieldMarkerIdent<'a> {
        FieldMarkerIdent {
            graphql_name: self.name.as_str(),
        }
    }
}

impl<'a> ObjectType<'a> {
    pub fn field_module(&self) -> FieldMarkerModule<'a> {
        FieldMarkerModule {
            type_name: self.name.clone(),
        }
    }
}

impl<'a> InterfaceType<'a> {
    pub fn field_module(&self) -> FieldMarkerModule<'a> {
        FieldMarkerModule {
            type_name: self.name.clone(),
        }
    }
}

impl<'a> InputObjectType<'a> {
    pub fn field_module(&'a self) -> FieldMarkerModule<'a> {
        FieldMarkerModule {
            type_name: self.name.clone(),
        }
    }
}

impl<'a> Field<'a> {
    pub fn argument_module(&self) -> ArgumentMarkerModule<'a> {
        ArgumentMarkerModule {
            type_name: self.parent_type_name.clone(),
            field_name: self.name.graphql_name.clone(),
        }
    }
}

impl<'a> ObjectRef<'a> {
    pub fn marker_ident(&self) -> TypeMarkerIdent<'a> {
        TypeMarkerIdent {
            graphql_name: self.0.clone(),
        }
    }
}

impl<'a> InterfaceRef<'a> {
    pub fn marker_ident(&self) -> TypeMarkerIdent<'a> {
        TypeMarkerIdent {
            graphql_name: self.0.clone(),
        }
    }
}

impl Directive<'_> {
    pub fn argument_module(&self) -> DirectiveArgumentMarkerModule<'_> {
        DirectiveArgumentMarkerModule {
            directive_name: self.name.borrow(),
        }
    }

    pub fn marker_ident(&self) -> DirectiveMarkerIdent<'_> {
        DirectiveMarkerIdent {
            graphql_name: self.name.borrow(),
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
