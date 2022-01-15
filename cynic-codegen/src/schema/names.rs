use std::borrow::Cow;

use crate::idents::{to_snake_case, RenableFieldIdent};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldName<'a> {
    pub(super) graphql_name: &'a str,
}

impl<'a> FieldName<'a> {
    pub fn as_str(&self) -> &'a str {
        self.graphql_name
    }
}

impl<'a> PartialEq<proc_macro2::Ident> for FieldName<'a> {
    fn eq(&self, other: &proc_macro2::Ident) -> bool {
        other == self.graphql_name
    }
}

impl<'a> PartialEq<str> for FieldName<'a> {
    fn eq(&self, other: &str) -> bool {
        self.graphql_name == other
    }
}

impl<'a> PartialEq<RenableFieldIdent> for FieldName<'a> {
    fn eq(&self, other: &RenableFieldIdent) -> bool {
        self.graphql_name == other.graphql_name()
    }
}

// impl From<&proc_macro2::Ident> for FieldName<'static> {
//     fn from(ident: &proc_macro2::Ident) -> Self {
//         FieldName {
//             graphql_name: Cow::Owned(to_snake_case(&ident.to_string())),
//         }
//     }
// }
