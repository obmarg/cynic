use darling::util::SpannedValue;

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(enum_unit))]
pub struct EnumDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) data: darling::ast::Data<EnumDeriveVariant, ()>,

    pub schema_path: SpannedValue<String>,
    pub graphql_type: SpannedValue<String>,
}

#[derive(Debug, darling::FromVariant)]
pub struct EnumDeriveVariant {
    pub(super) ident: proc_macro2::Ident,
}

/// An alternative EnumDeriveInput struct that doesn't require as many fields.
///
/// This is used by the query_module generation, which provides some of the parameters
/// without the users input.
#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(struct_named))]
pub(crate) struct QueryModuleEnumDeriveInput {
    ident: proc_macro2::Ident,
    data: darling::ast::Data<EnumDeriveVariant, ()>,

    #[darling(default)]
    pub schema_path: Option<SpannedValue<String>>,
    pub graphql_type: SpannedValue<String>,
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
        }
    }
}
