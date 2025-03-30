use std::borrow::Cow;

use crate::idents::RenamableFieldIdent;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct FieldName<'a> {
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
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

impl PartialEq<proc_macro2::Ident> for FieldName<'_> {
    fn eq(&self, other: &proc_macro2::Ident) -> bool {
        other == self.graphql_name.as_ref()
    }
}

impl PartialEq<str> for FieldName<'_> {
    fn eq(&self, other: &str) -> bool {
        self.graphql_name == other
    }
}

impl PartialEq<String> for FieldName<'_> {
    fn eq(&self, other: &String) -> bool {
        self.graphql_name.as_ref() == other
    }
}

impl PartialEq<RenamableFieldIdent> for FieldName<'_> {
    fn eq(&self, other: &RenamableFieldIdent) -> bool {
        self.graphql_name == other.graphql_name()
    }
}
