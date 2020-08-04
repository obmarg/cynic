#[derive(darling::FromDeriveInput)]
#[darling(supports(struct_newtype))]
pub struct ScalarDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) data: darling::ast::Data<(), ScalarDeriveField>,
}

#[derive(darling::FromField)]
#[darling(forward_attrs(arguments))]
pub struct ScalarDeriveField {
    pub(super) ty: syn::Type,
}
