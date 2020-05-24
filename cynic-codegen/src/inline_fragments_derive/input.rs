use darling::util::SpannedValue;

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(enum_newtype))]
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
}

#[derive(darling::FromField)]
#[darling(attributes(cynic))]
pub(super) struct InlineFragmentsDeriveField {
    pub ty: syn::Type,
}

/// An alternative FragmentDeriveInput struct that doesn't require as many fields.
///
/// This is used by the query_module generation, which provides some of the parameters
/// without the users input.
#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(enum_newtype))]
pub(crate) struct QueryModuleInlineFragmentsDeriveInput {
    ident: proc_macro2::Ident,
    data: darling::ast::Data<SpannedValue<InlineFragmentsDeriveVariant>, ()>,

    #[darling(default)]
    pub schema_path: Option<SpannedValue<String>>,
    #[darling(default)]
    pub query_module: Option<SpannedValue<String>>,
    pub graphql_type: SpannedValue<String>,
    #[darling(default)]
    pub argument_struct: Option<syn::Ident>,
}

impl QueryModuleInlineFragmentsDeriveInput {
    pub fn to_inline_fragments_derive_input(
        self,
        schema_path: &SpannedValue<String>,
        query_module: &SpannedValue<String>,
    ) -> InlineFragmentsDeriveInput {
        InlineFragmentsDeriveInput {
            ident: self.ident,
            data: self.data,
            schema_path: self.schema_path.unwrap_or_else(|| schema_path.clone()),
            query_module: self.query_module.unwrap_or_else(|| query_module.clone()),
            graphql_type: self.graphql_type,
            argument_struct: self.argument_struct,
        }
    }
}
