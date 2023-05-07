use std::borrow::Cow;

use crate::idents::RenamableFieldIdent;

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct FieldName<'a> {
    #[with(rkyv::with::AsOwned)]
    pub(super) graphql_name: Cow<'a, str>,
}

impl<'a> FieldName<'a> {
    pub fn new(graphql_name: &'a str) -> Self {
        FieldName {
            graphql_name: Cow::Borrowed(graphql_name),
        }
    }

    pub fn as_str(&'a self) -> &'a str {
        self.graphql_name.as_ref()
    }

    pub fn to_literal(&self) -> proc_macro2::Literal {
        proc_macro2::Literal::string(self.graphql_name.as_ref())
    }
}

impl<'a> PartialEq<proc_macro2::Ident> for FieldName<'a> {
    fn eq(&self, other: &proc_macro2::Ident) -> bool {
        other == self.graphql_name.as_ref()
    }
}

impl<'a> PartialEq<str> for FieldName<'a> {
    fn eq(&self, other: &str) -> bool {
        self.graphql_name == other
    }
}

impl<'a> PartialEq<String> for FieldName<'a> {
    fn eq(&self, other: &String) -> bool {
        self.graphql_name.as_ref() == other
    }
}

impl<'a> PartialEq<RenamableFieldIdent> for FieldName<'a> {
    fn eq(&self, other: &RenamableFieldIdent) -> bool {
        self.graphql_name == other.graphql_name()
    }
}
