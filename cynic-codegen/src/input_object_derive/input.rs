use darling::util::SpannedValue;

use crate::ident::RenameAll;
use proc_macro2::Span;

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(struct_named))]
pub struct InputObjectDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) data: darling::ast::Data<(), InputObjectDeriveField>,

    pub schema_path: SpannedValue<String>,
    pub query_module: SpannedValue<String>,

    #[darling(default)]
    pub graphql_type: Option<SpannedValue<String>>,

    #[darling(default)]
    pub require_all_fields: bool,

    #[darling(default)]
    pub(super) rename_all: Option<RenameAll>,
}

#[derive(Debug, darling::FromField)]
#[darling(attributes(cynic))]
pub struct InputObjectDeriveField {
    pub(super) ident: Option<proc_macro2::Ident>,
    pub(super) ty: syn::Type,

    #[darling(default)]
    pub(super) skip_serializing_if: Option<SpannedValue<syn::Path>>,

    #[darling(default)]
    pub(super) rename: Option<SpannedValue<String>>,
}

impl InputObjectDeriveInput {
    pub fn graphql_type_name(&self) -> String {
        self.graphql_type
            .as_ref()
            .map(|sp| sp.to_string())
            .unwrap_or_else(|| self.ident.to_string())
    }

    pub fn graphql_type_span(&self) -> Span {
        self.graphql_type
            .as_ref()
            .map(|val| val.span())
            .unwrap_or_else(|| self.ident.span())
    }
}
