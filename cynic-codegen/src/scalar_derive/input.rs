use darling::util::SpannedValue;

#[derive(darling::FromDeriveInput)]
#[darling(supports(struct_newtype))]
pub struct ScalarDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) data: darling::ast::Data<(), ScalarDeriveField>,
}

#[derive(darling::FromField)]
#[darling(forward_attrs(cynic_arguments))]
pub struct ScalarDeriveField {
    pub(super) ty: syn::Type,
}

/// An alternative ScalarDeriveInput struct that doesn't require as many fields.
///
/// This is used by the query_module generation, which provides some of the parameters
/// without the users input.
#[derive(darling::FromDeriveInput)]
#[darling(supports(struct_newtype))]
pub(crate) struct QueryModuleScalarDeriveInput {
    ident: proc_macro2::Ident,
    data: darling::ast::Data<(), crate::scalar_derive::ScalarDeriveField>,
}

impl QueryModuleScalarDeriveInput {
    pub fn to_scalar_derive_input(
        self,
        schema_path: &SpannedValue<String>,
        query_module: &SpannedValue<String>,
    ) -> ScalarDeriveInput {
        ScalarDeriveInput {
            ident: self.ident,
            data: self.data,
        }
    }
}
