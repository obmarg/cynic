use darling::util::SpannedValue;

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(enum_newtype, enum_unit))]
pub struct InlineFragmentsDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) data: darling::ast::Data<SpannedValue<InlineFragmentsDeriveVariant>, ()>,

    pub schema_path: SpannedValue<String>,
    pub query_module: SpannedValue<String>,
    pub graphql_type: SpannedValue<String>,
    #[darling(default)]
    pub argument_struct: Option<syn::Ident>,
}

#[derive(darling::FromVariant)]
#[darling(attributes(cynic))]
pub(super) struct InlineFragmentsDeriveVariant {
    pub ident: proc_macro2::Ident,
    pub fields: darling::ast::Fields<InlineFragmentsDeriveField>,

    #[darling(default)]
    pub(super) fallback: SpannedValue<bool>,
}

#[derive(darling::FromField)]
#[darling(attributes(cynic))]
pub(super) struct InlineFragmentsDeriveField {
    pub ty: syn::Type,
}
