use darling::util::SpannedValue;
use proc_macro2::Span;

use crate::idents::{RenamableFieldIdent, RenameAll};

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(enum_unit))]
pub struct EnumDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) data: darling::ast::Data<EnumDeriveVariant, ()>,

    pub schema_path: SpannedValue<String>,

    #[darling(default, rename = "schema_module")]
    schema_module_: Option<syn::Path>,

    #[darling(default)]
    pub graphql_type: Option<SpannedValue<String>>,

    #[darling(default)]
    pub(super) rename_all: Option<RenameAll>,
}

impl EnumDeriveInput {
    pub fn schema_module(&self) -> syn::Path {
        if let Some(schema_module) = &self.schema_module_ {
            return schema_module.clone();
        }
        syn::parse2(quote::quote! { schema }).unwrap()
    }
}

#[derive(Debug, darling::FromVariant)]
#[darling(attributes(cynic))]
pub struct EnumDeriveVariant {
    pub(super) ident: proc_macro2::Ident,

    #[darling(default)]
    rename: Option<SpannedValue<String>>,
}

impl EnumDeriveInput {
    pub fn graphql_type_name(&self) -> String {
        self.graphql_type
            .as_ref()
            .map(|sp| sp.to_string())
            .unwrap_or_else(|| self.ident.to_string())
    }

    pub fn graphql_type_span(&self) -> Span {
        self.graphql_type
            .as_ref()
            .map(|val| val.span())
            .unwrap_or_else(|| self.ident.span())
    }
}

impl EnumDeriveVariant {
    pub(super) fn graphql_ident(&self, rename_rule: RenameAll) -> RenamableFieldIdent {
        let mut ident = RenamableFieldIdent::from(self.ident.clone());
        match &self.rename {
            Some(rename) => {
                let span = rename.span();
                let rename = (**rename).clone();
                ident.set_rename(rename, span)
            }
            None => {
                ident.rename_with(rename_rule, self.ident.span());
            }
        }
        ident
    }
}
