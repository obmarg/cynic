use darling::util::SpannedValue;
use proc_macro2::Span;

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(enum_newtype, enum_unit))]
pub struct InlineFragmentsDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) data: darling::ast::Data<SpannedValue<InlineFragmentsDeriveVariant>, ()>,

    pub schema_path: SpannedValue<String>,

    // query_module is deprecated, remove eventually.
    #[darling(default)]
    query_module: Option<SpannedValue<String>>,
    #[darling(default, rename = "schema_module")]
    schema_module_: Option<syn::Path>,

    #[darling(default)]
    pub graphql_type: Option<SpannedValue<String>>,
    #[darling(default)]
    pub argument_struct: Option<syn::Ident>,
}

impl InlineFragmentsDeriveInput {
    pub fn schema_module(&self) -> syn::Path {
        if let Some(schema_module) = &self.schema_module_ {
            return schema_module.clone();
        }
        if let Some(query_module) = &self.query_module {
            return syn::parse_str(query_module).unwrap();
        }
        syn::parse2(quote::quote! { schema }).unwrap()
    }

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

#[derive(darling::FromVariant)]
#[darling(attributes(cynic))]
pub(super) struct InlineFragmentsDeriveVariant {
    pub(super) ident: proc_macro2::Ident,
    pub fields: darling::ast::Fields<InlineFragmentsDeriveField>,

    #[darling(default)]
    rename: Option<SpannedValue<String>>,

    #[darling(default)]
    pub(super) fallback: SpannedValue<bool>,
}

#[derive(darling::FromField)]
#[darling(attributes(cynic))]
pub(super) struct InlineFragmentsDeriveField {
    pub ty: syn::Type,
}

impl InlineFragmentsDeriveVariant {
    pub(super) fn graphql_ident(&self) -> crate::Ident {
        if let Some(rename) = &self.rename {
            return crate::Ident::for_type(&**rename).with_span(rename.span());
        }

        crate::Ident::from_proc_macro2(&self.ident, None)
    }
}
