use darling::util::SpannedValue;
use proc_macro2::Span;
use quote::quote_spanned;

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

    // argument_struct is deprecated, remove eventually.
    #[darling(default)]
    argument_struct: Option<syn::Ident>,
    #[darling(default)]
    variables: Option<syn::Path>,
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

    pub fn variables(&self) -> Option<syn::Path> {
        self.variables
            .clone()
            .or_else(|| self.argument_struct.clone().map(Into::into))
    }

    pub fn deprecations(&self) -> proc_macro2::TokenStream {
        if self.variables.is_none() && self.argument_struct.is_some() {
            let span = self.argument_struct.as_ref().map(|x| x.span()).unwrap();
            return quote_spanned! { span =>
                #[allow(clippy::no_effect, non_camel_case_types)]
                const _: fn() = || {
                    #[deprecated(note = "the argument_struct attribute is deprecated.  use the variables attribute instead", since = "2.0.0")]
                    struct argument_struct {}
                    argument_struct {};
                };
            };
        }

        proc_macro2::TokenStream::new()
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
    pub(super) fn graphql_name(&self) -> String {
        self.rename
            .as_deref()
            .cloned()
            .unwrap_or_else(|| self.ident.to_string())
    }
}
