use darling::util::SpannedValue;
use proc_macro2::Span;

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

impl EnumDeriveInput {
    pub fn graphql_type_name(&self) -> String {
        // May be have a good way.
        String::from(
            &*(self
                .graphql_type
                .as_ref()
                .map(|val| val.clone())
                .unwrap_or(SpannedValue::from(self.ident.to_string()))),
        )
    }

    pub fn graphql_type_span(&self) -> Span {
        self.graphql_type
            .as_ref()
            .map(|val| val.span())
            .unwrap_or(self.ident.span())
    }
}
