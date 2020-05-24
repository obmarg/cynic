use crate::load_schema;
use darling::util::SpannedValue;

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(struct_named))]
pub struct FragmentDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) data: darling::ast::Data<(), FragmentDeriveField>,

    pub schema_path: SpannedValue<String>,
    pub query_module: SpannedValue<String>,
    pub graphql_type: SpannedValue<String>,
    #[darling(default)]
    pub argument_struct: Option<syn::Ident>,
}

#[derive(darling::FromField)]
#[darling(attributes(cynic), forward_attrs(cynic_arguments))]
pub struct FragmentDeriveField {
    pub(super) ident: Option<proc_macro2::Ident>,
    pub(super) ty: syn::Type,

    pub(super) attrs: Vec<syn::Attribute>,

    #[darling(default)]
    pub(super) flatten: bool,
}

/// An alternative FragmentDeriveInput struct that doesn't require as many fields.
///
/// This is used by the query_module generation, which provides some of the parameters
/// without the users input.
#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(struct_named))]
pub(crate) struct QueryModuleFragmentDeriveInput {
    ident: proc_macro2::Ident,
    data: darling::ast::Data<(), crate::fragment_derive::FragmentDeriveField>,

    #[darling(default)]
    pub schema_path: Option<SpannedValue<String>>,
    #[darling(default)]
    pub query_module: Option<SpannedValue<String>>,
    pub graphql_type: SpannedValue<String>,
    #[darling(default)]
    pub argument_struct: Option<syn::Ident>,
}

impl QueryModuleFragmentDeriveInput {
    pub fn to_fragment_derive_input(
        self,
        schema_path: &SpannedValue<String>,
        query_module: &SpannedValue<String>,
    ) -> FragmentDeriveInput {
        FragmentDeriveInput {
            ident: self.ident,
            data: self.data,
            schema_path: self.schema_path.unwrap_or_else(|| schema_path.clone()),
            query_module: self.query_module.unwrap_or_else(|| query_module.clone()),
            graphql_type: self.graphql_type,
            argument_struct: self.argument_struct,
        }
    }
}
