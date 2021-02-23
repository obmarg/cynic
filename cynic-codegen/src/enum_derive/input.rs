use darling::util::SpannedValue;

use crate::ident::RenameAll;

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(enum_unit))]
pub struct EnumDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) data: darling::ast::Data<EnumDeriveVariant, ()>,

    pub schema_path: SpannedValue<String>,
    pub query_module: SpannedValue<String>,

    #[darling(default)]
    pub graphql_type: Option<SpannedValue<String>>,

    #[darling(default)]
    pub(super) rename_all: Option<RenameAll>,
}

#[derive(Debug, darling::FromVariant)]
#[darling(attributes(cynic))]
pub struct EnumDeriveVariant {
    pub(super) ident: proc_macro2::Ident,

    #[darling(default)]
    pub(super) rename: Option<SpannedValue<String>>,
}
