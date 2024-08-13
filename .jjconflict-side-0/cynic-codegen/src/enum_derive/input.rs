use darling::util::SpannedValue;
use proc_macro2::Span;

use crate::{
    error::Errors,
    idents::{RenamableFieldIdent, RenameAll},
    schema::SchemaInput,
};

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(enum_unit, enum_newtype))]
pub struct EnumDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) data: darling::ast::Data<SpannedValue<EnumDeriveVariant>, ()>,

    #[darling(default)]
    schema: Option<SpannedValue<String>>,
    #[darling(default)]
    schema_path: Option<SpannedValue<String>>,

    #[darling(default, rename = "schema_module")]
    schema_module_: Option<syn::Path>,

    #[darling(default)]
    pub graphql_type: Option<SpannedValue<String>>,

    #[darling(default)]
    pub(super) rename_all: Option<RenameAll>,

    #[darling(default)]
    pub(super) non_exhaustive: bool,
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
    pub(super) rename: Option<SpannedValue<String>>,

    #[darling(default)]
    pub(super) fallback: SpannedValue<bool>,

    pub(super) fields: darling::ast::Fields<()>,
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

    pub fn schema_input(&self) -> Result<SchemaInput, syn::Error> {
        match (&self.schema, &self.schema_path) {
            (None, None) => SchemaInput::default().map_err(|e| e.into_syn_error(Span::call_site())),
            (None, Some(path)) => SchemaInput::from_schema_path(path.as_ref())
                .map_err(|e| e.into_syn_error(path.span())),
            (Some(name), None) => SchemaInput::from_schema_name(name.as_ref())
                .map_err(|e| e.into_syn_error(name.span())),
            (Some(_), Some(path)) => Err(syn::Error::new(
                path.span(),
                "Only one of schema_path & schema can be provided",
            )),
        }
    }

    pub(super) fn validate(&self) -> Result<(), Errors> {
        let data_ref = self.data.as_ref().take_enum().unwrap();
        let fallbacks = data_ref.iter().filter(|v| *v.fallback).collect::<Vec<_>>();
        let mut errors = Errors::default();

        if fallbacks.len() > 1 {
            errors.extend(
                fallbacks
                    .into_iter()
                    .map(|f| {
                        syn::Error::new(
                            f.span(),
                            "Enums only support a single fallback, but this enum has many",
                        )
                    })
                    .collect::<Vec<_>>(),
            );
        }

        errors.extend(data_ref.iter().filter_map(|v| v.validate(v.span()).err()));

        errors.into_result(())
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

    fn validate(&self, span: proc_macro2::Span) -> Result<(), Errors> {
        use darling::ast::Style::*;

        if *self.fallback {
            match (self.fields.style, self.fields.len()) {
                (Unit, _) => Ok(()),
                (Struct, _) => Err(syn::Error::new(
                    span,
                    "Enum derive doesn't support struct variants as a fallback",
                )
                .into()),
                (Tuple, 1) => Ok(()),
                (Tuple, _) => Err(syn::Error::new(
                    span,
                    "Enum derive fallbacks can only have a single field",
                )
                .into()),
            }
            // TODO: make sure we only have a string somewhere
        } else {
            match self.fields.style {
                Unit => Ok(()),
                _ => Err(syn::Error::new(span, "GraphQL Enums can't have fields").into()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use darling::FromDeriveInput;
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_enum_with_no_fallbacks() {
        let input = EnumDeriveInput::from_derive_input(&parse_quote! {
            enum TestEnum {
                Foo,
                Bar
            }
        })
        .unwrap();

        input.validate().expect("no errors");
    }

    #[test]
    fn test_enum_with_empty_fallback() {
        let input = EnumDeriveInput::from_derive_input(&parse_quote! {
            enum TestEnum {
                Foo,
                #[cynic(fallback)]
                Bar
            }
        })
        .unwrap();

        input.validate().expect("no errors");
    }

    #[test]
    fn test_enum_with_string_fallback() {
        let input = EnumDeriveInput::from_derive_input(&parse_quote! {
            enum TestEnum {
                Foo,
                #[cynic(fallback)]
                Bar(String)
            }
        })
        .unwrap();

        input.validate().expect("no errors");
    }
}
