use quote::format_ident;

use crate::{idents::PathExt2, schema::types::*};

use super::{to_pascal_case, to_snake_case};

// TODO: Not sure this really lives in the schema module.  Probably doesn't tbh
// Although it is extending the schema types, so who knows...

#[derive(Clone, Copy)]
pub struct MarkerIdent<'a> {
    graphql_name: &'a str,
}

#[derive(Clone, Copy)]
pub struct FieldMarkerIdent<'a> {
    graphql_name: &'a str,
}

#[derive(Clone, Copy)]
pub struct FieldMarkerModule<'a> {
    type_name: &'a str,
}

#[derive(Clone, Copy)]
pub struct ArgumentMarkerModule<'a> {
    type_name: &'a str,
    field_name: &'a str,
}

impl<'a> MarkerIdent<'a> {
    pub fn located_at_path(self, path_to_markers: &syn::Path) -> syn::Path {
        let mut path = path_to_markers.clone();
        path.push(format_ident!("{}", to_pascal_case(self.graphql_name)));
        path
    }
}

impl<'a> FieldMarkerModule<'a> {
    pub fn located_at_path(self, path_to_markers: &syn::Path) -> syn::Path {
        let mut path = path_to_markers.clone();
        path.push(format_ident!("{}_fields", to_snake_case(self.type_name)));
        path
    }
}

impl<'a> FieldMarkerIdent<'a> {
    pub fn located_at_path(self, path_to_markers: &syn::Path) -> syn::Path {
        let mut path = path_to_markers.clone();
        path.push(format_ident!("{}", to_pascal_case(self.graphql_name)));
        path
    }
}

macro_rules! marker_ident_for_named {
    () => {};
    ($kind:ident) => {
        impl<'a> $kind<'a> {
            pub fn marker_ident(&self) -> MarkerIdent<'a> {
                MarkerIdent {
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
    InputObjectType
);

impl<'a> Field<'a> {
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

// TODO: Probably need some kind of wrapper type like
// IdentTokens(schema_path, MarkerIdent)
// Maybe Ident could become a trait?
