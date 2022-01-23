use darling::util::SpannedValue;
use syn::spanned::Spanned;

use crate::idents::{RenamableFieldIdent, RenameAll};
use proc_macro2::Span;

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(struct_named))]
pub struct FragmentArgumentsDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) vis: syn::Visibility,
    pub(super) data: darling::ast::Data<(), FragmentArgumentsField>,

    #[darling(default)]
    schema_module: Option<syn::Path>,

    #[darling(default)]
    pub(super) rename_all: Option<RenameAll>,
}

#[derive(Debug, darling::FromField)]
#[darling(attributes(cynic))]
pub(super) struct FragmentArgumentsField {
    pub(super) ident: Option<proc_macro2::Ident>,
    pub(super) ty: syn::Type,

    #[darling(default)]
    pub(super) rename: Option<SpannedValue<String>>,
}

impl FragmentArgumentsDeriveInput {
    pub fn schema_module(&self) -> syn::Path {
        if let Some(schema_module) = &self.schema_module {
            return schema_module.clone();
        }
        syn::parse2(quote::quote! { schema }).unwrap()
    }
}

impl FragmentArgumentsField {
    pub fn graphql_ident(&self, rename_all: Option<RenameAll>) -> RenamableFieldIdent {
        let mut ident = RenamableFieldIdent::from(
            self.ident
                .clone()
                .expect("InputObject only supports named structs"),
        );
        if let Some(rename) = &self.rename {
            let span = rename.span();
            let rename = (**rename).clone();
            ident.set_rename(rename, span)
        } else if let Some(rename_all) = rename_all {
            ident.rename_with(rename_all, self.ident.span())
        }
        ident
    }
}
