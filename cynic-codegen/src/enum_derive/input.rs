use darling::util::SpannedValue;

use crate::ident::RenameAll;

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(enum_unit))]
pub struct EnumDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) data: darling::ast::Data<EnumDeriveVariant, ()>,

    pub schema_path: SpannedValue<String>,
    pub graphql_type: SpannedValue<String>,

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

/// An alternative EnumDeriveInput struct that doesn't require as many fields.
///
/// This is used by the query_module generation, which provides some of the parameters
/// without the users input.
#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(enum_unit))]
pub(crate) struct QueryModuleEnumDeriveInput {
    ident: proc_macro2::Ident,
    data: darling::ast::Data<EnumDeriveVariant, ()>,

    #[darling(default)]
    pub schema_path: Option<SpannedValue<String>>,
    pub graphql_type: SpannedValue<String>,

    #[darling(default)]
    rename_all: Option<RenameAll>,
}

impl QueryModuleEnumDeriveInput {
    pub fn to_enum_derive_input(
        self,
        schema_path: &SpannedValue<String>,
        query_module: &SpannedValue<String>,
    ) -> EnumDeriveInput {
        EnumDeriveInput {
            ident: self.ident,
            data: self.data,
            schema_path: self.schema_path.unwrap_or_else(|| schema_path.clone()),
            graphql_type: self.graphql_type,
            rename_all: self.rename_all,
        }
    }
}
