use darling::util::SpannedValue;

use crate::{type_validation::CheckMode, Errors};
use proc_macro2::Span;

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(struct_named))]
pub struct FragmentDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) data: darling::ast::Data<(), FragmentDeriveField>,

    pub schema_path: SpannedValue<String>,
    pub query_module: SpannedValue<String>,

    #[darling(default)]
    pub graphql_type: Option<SpannedValue<String>>,
    #[darling(default)]
    pub argument_struct: Option<syn::Ident>,
}

impl FragmentDeriveInput {
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

    pub fn validate(&self) -> Result<(), Errors> {
        let data_field_is_empty = matches!(self.data.clone(), darling::ast::Data::Struct(fields) if fields.fields.is_empty());
        if data_field_is_empty {
            return Err(syn::Error::new(
                self.ident.span(),
                format!(
                    "At least one field should be selected for `{}`.",
                    self.ident.to_string()
                ),
            )
            .into());
        }

        let errors = self
            .data
            .clone()
            .map_struct_fields(|field| field.validate().err())
            .take_struct()
            .unwrap()
            .into_iter()
            .flatten()
            .collect::<Errors>();

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }
}

#[derive(darling::FromField, Clone)]
#[darling(attributes(cynic), forward_attrs(arguments))]
pub struct FragmentDeriveField {
    pub(super) ident: Option<proc_macro2::Ident>,
    pub(super) ty: syn::Type,

    pub(super) attrs: Vec<syn::Attribute>,

    #[darling(default)]
    pub(super) flatten: bool,

    #[darling(default)]
    pub(super) recurse: Option<SpannedValue<u8>>,
}

impl FragmentDeriveField {
    pub fn validate(&self) -> Result<(), Errors> {
        if self.flatten && self.recurse.is_some() {
            return Err(syn::Error::new(
                self.recurse.as_ref().unwrap().span(),
                "A field can't be recurse if it's being flattened",
            )
            .into());
        }

        Ok(())
    }

    pub fn type_check_mode(&self) -> CheckMode {
        if self.flatten {
            CheckMode::Flattening
        } else if self.recurse.is_some() {
            CheckMode::Recursing
        } else {
            CheckMode::Normal
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_matches::assert_matches;
    use quote::format_ident;

    #[test]
    fn test_fragment_derive_validate_pass() {
        let input = FragmentDeriveInput {
            ident: format_ident!("TestInput"),
            data: darling::ast::Data::Struct(darling::ast::Fields {
                style: darling::ast::Style::Struct,
                fields: vec![
                    FragmentDeriveField {
                        ident: Some(format_ident!("field_one")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false,
                        recurse: None,
                    },
                    FragmentDeriveField {
                        ident: Some(format_ident!("field_two")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: true,
                        recurse: None,
                    },
                    FragmentDeriveField {
                        ident: Some(format_ident!("field_three")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false,
                        recurse: Some(8.into()),
                    },
                ],
            }),
            schema_path: "abcd".to_string().into(),
            query_module: "abcd".to_string().into(),
            graphql_type: Some("abcd".to_string().into()),
            argument_struct: None,
        };

        assert_matches!(input.validate(), Ok(()));
    }

    #[test]
    fn test_fragment_derive_validate_fails() {
        let input = FragmentDeriveInput {
            ident: format_ident!("TestInput"),
            data: darling::ast::Data::Struct(darling::ast::Fields {
                style: darling::ast::Style::Struct,
                fields: vec![
                    FragmentDeriveField {
                        ident: Some(format_ident!("field_one")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false,
                        recurse: None,
                    },
                    FragmentDeriveField {
                        ident: Some(format_ident!("field_two")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: true,
                        recurse: Some(8.into()),
                    },
                    FragmentDeriveField {
                        ident: Some(format_ident!("field_three")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: true,
                        recurse: Some(8.into()),
                    },
                ],
            }),
            schema_path: "abcd".to_string().into(),
            query_module: "abcd".to_string().into(),
            graphql_type: Some("abcd".to_string().into()),
            argument_struct: None,
        };

        let errors = input.validate().unwrap_err();
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_fragment_derive_validate_failed() {
        let input = FragmentDeriveInput {
            ident: format_ident!("TestInput"),
            data: darling::ast::Data::Struct(darling::ast::Fields {
                style: darling::ast::Style::Struct,
                fields: vec![],
            }),
            schema_path: "abcd".to_string().into(),
            query_module: "abcd".to_string().into(),
            graphql_type: Some("abcd".to_string().into()),
            argument_struct: None,
        };
        let errors = input.validate().unwrap_err();
        assert_eq!(
            errors.to_compile_errors().to_string(),
            r#"compile_error ! { "At least one field should be selected for `TestInput`." }"#
                .to_string()
        );
    }

    #[test]
    fn test_fragment_derive_validate_pass_no_graphql_type() {
        let input = FragmentDeriveInput {
            ident: format_ident!("TestInput"),
            data: darling::ast::Data::Struct(darling::ast::Fields {
                style: darling::ast::Style::Struct,
                fields: vec![
                    FragmentDeriveField {
                        ident: Some(format_ident!("field_one")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false,
                        recurse: None,
                    },
                    FragmentDeriveField {
                        ident: Some(format_ident!("field_two")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: true,
                        recurse: None,
                    },
                    FragmentDeriveField {
                        ident: Some(format_ident!("field_three")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false,
                        recurse: Some(8.into()),
                    },
                ],
            }),
            schema_path: "abcd".to_string().into(),
            query_module: "abcd".to_string().into(),
            graphql_type: None,
            argument_struct: None,
        };

        assert_matches!(input.validate(), Ok(()));
    }
}
