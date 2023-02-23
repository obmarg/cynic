use darling::util::SpannedValue;

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(struct_newtype))]
pub struct ScalarDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) data: darling::ast::Data<(), ScalarDeriveField>,
    pub(super) generics: syn::Generics,

    #[darling(default, rename = "schema_module")]
    schema_module_: Option<syn::Path>,

    #[darling(default)]
    pub(super) graphql_type: Option<SpannedValue<String>>,
}

#[derive(darling::FromField)]
#[darling(forward_attrs(arguments))]
pub struct ScalarDeriveField {
    pub(super) ty: syn::Type,
}

impl ScalarDeriveInput {
    pub fn schema_module(&self) -> syn::Path {
        if let Some(schema_module) = &self.schema_module_ {
            return schema_module.clone();
        }
        syn::parse2(quote::quote! { schema }).unwrap()
    }
}
