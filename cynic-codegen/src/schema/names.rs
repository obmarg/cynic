use crate::idents::RenamableFieldIdent;

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

impl<'a> PartialEq<RenamableFieldIdent> for FieldName<'a> {
    fn eq(&self, other: &RenamableFieldIdent) -> bool {
        self.graphql_name == other.graphql_name()
    }
}
