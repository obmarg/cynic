use darling::util::SpannedValue;
use syn::spanned::Spanned;

use crate::{idents::RenamableFieldIdent, RenameAll};

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(struct_named))]
pub struct QueryVariableLiteralsInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) generics: syn::Generics,
    pub(super) data: darling::ast::Data<(), QueryVariableLiteralsField>,

    #[darling(default)]
    pub(super) rename_all: Option<RenameAll>,
}

#[derive(Debug, darling::FromField)]
#[darling(attributes(cynic))]
pub(super) struct QueryVariableLiteralsField {
    pub(super) ident: Option<proc_macro2::Ident>,

    #[darling(default)]
    pub(super) skip_serializing_if: Option<SpannedValue<syn::Path>>,

    #[darling(default)]
    pub(super) rename: Option<SpannedValue<String>>,
}

impl QueryVariableLiteralsField {
    pub fn graphql_ident(&self, rename_all: Option<RenameAll>) -> RenamableFieldIdent {
        let mut ident = RenamableFieldIdent::from(
            self.ident
                .clone()
                .expect("InputObject only supports named structs"),
        );
        if let Some(rename) = &self.rename {
            let span = rename.span();
            let rename = (**rename).clone();
            ident.set_rename(rename, span)
        } else if let Some(rename_all) = rename_all {
            ident.rename_with(rename_all, self.ident.span())
        }
        ident
    }
}
