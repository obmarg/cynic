use darling::util::SpannedValue;
use proc_macro2::Span;

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(struct_newtype))]
pub struct ScalarDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) data: darling::ast::Data<(), ScalarDeriveField>,

    #[darling(default, rename = "schema_module")]
    schema_module_: Option<SpannedValue<String>>,

    #[darling(default)]
    pub(super) graphql_type: Option<SpannedValue<String>>,
}

#[derive(darling::FromField)]
#[darling(forward_attrs(arguments))]
pub struct ScalarDeriveField {
    pub(super) ty: syn::Type,
}

impl ScalarDeriveInput {
    pub fn schema_module(&self) -> SpannedValue<String> {
        if let Some(schema_module) = &self.schema_module_ {
            return schema_module.clone();
        }

        SpannedValue::new("schema".into(), Span::call_site())
    }
}
