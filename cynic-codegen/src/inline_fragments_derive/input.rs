use darling::util::SpannedValue;
use proc_macro2::Span;

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(enum_newtype, enum_unit))]
pub struct InlineFragmentsDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) data: darling::ast::Data<SpannedValue<InlineFragmentsDeriveVariant>, ()>,

    pub schema_path: SpannedValue<String>,
    pub query_module: SpannedValue<String>,

    #[darling(default)]
    pub graphql_type: Option<SpannedValue<String>>,
    #[darling(default)]
    pub argument_struct: Option<syn::Ident>,
}

impl InlineFragmentsDeriveInput {
    pub fn graphql_type_name(&self) -> String {
        self.graphql_type
            .as_ref()
            .map(|sp| String::from((*sp).to_string()))
            .unwrap_or_else(|| self.ident.to_string())
    }

    pub fn graphql_type_span(&self) -> Span {
        self.graphql_type
            .as_ref()
            .map(|val| val.span())
            .unwrap_or_else(|| self.ident.span())
    }
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
