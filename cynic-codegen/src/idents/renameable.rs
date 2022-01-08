use super::{to_camel_case, to_pascal_case};

/// A wrapper around proc_macro2::Ident for a struct field that keeps
/// track of whether the given field needs renamed to map to a graphql
/// field.
pub struct RenableFieldIdent {
    ident: proc_macro2::Ident,
    renamed: Option<(String, proc_macro2::Span)>,
}

impl From<proc_macro2::Ident> for RenableFieldIdent {
    fn from(ident: proc_macro2::Ident) -> Self {
        RenableFieldIdent {
            ident,
            renamed: None,
        }
    }
}

impl RenableFieldIdent {
    pub fn set_rename(&mut self, rename: String, rename_span: proc_macro2::Span) {
        self.renamed = Some((rename, rename_span));
    }

    pub fn graphql_name(&self) -> String {
        if let Some((rename, _)) = &self.renamed {
            return rename.clone();
        }

        to_camel_case(&self.ident.to_string())
    }

    pub fn span(&self) -> proc_macro2::Span {
        if let Some((_, span)) = &self.renamed {
            return *span;
        }

        return self.ident.span();
    }
}
