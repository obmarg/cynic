use darling::util::SpannedValue;

use crate::{type_validation::CheckMode, Errors};
use proc_macro2::Span;

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(struct_named))]
pub struct FragmentDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) data: darling::ast::Data<(), FragmentDeriveField>,

    pub schema_path: SpannedValue<String>,

    // query_module is deprecated, remove eventually.
    #[darling(default)]
    query_module: Option<SpannedValue<String>>,
    #[darling(default, rename = "schema_module")]
    schema_module_: Option<SpannedValue<String>>,

    #[darling(default)]
    pub graphql_type: Option<SpannedValue<String>>,
    #[darling(default)]
    pub argument_struct: Option<syn::Ident>,
}

impl FragmentDeriveInput {
    pub fn schema_module(&self) -> SpannedValue<String> {
        if let Some(schema_module) = &self.schema_module_ {
            return schema_module.clone();
        }
        if let Some(query_module) = &self.query_module {
            return query_module.clone();
        }

        SpannedValue::new("schema".into(), Span::call_site())
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
    pub(super) flatten: SpannedValue<bool>,

    #[darling(default)]
    pub(super) recurse: Option<SpannedValue<u8>>,

    #[darling(default)]
    pub(super) spread: SpannedValue<bool>,

    #[darling(default)]
    rename: Option<SpannedValue<String>>,
}

impl FragmentDeriveField {
    pub fn validate(&self) -> Result<(), Errors> {
        if *self.flatten && self.recurse.is_some() {
            return Err(syn::Error::new(
                self.recurse.as_ref().unwrap().span(),
                "A field can't be recurse if it's being flattened",
            )
            .into());
        }

        if *self.flatten && *self.spread {
            return Err(syn::Error::new(
                self.flatten.span(),
                "A field can't be flattened if it's also being spread",
            )
            .into());
        }

        if *self.spread && self.recurse.is_some() {
            return Err(syn::Error::new(
                self.recurse.as_ref().unwrap().span(),
                "A field can't be recurse if it's being spread",
            )
            .into());
        }

        Ok(())
    }

    pub fn type_check_mode(&self) -> CheckMode {
        if *self.flatten {
            CheckMode::Flattening
        } else if self.recurse.is_some() {
            CheckMode::Recursing
        } else if *self.spread {
            CheckMode::Spreading
        } else {
            CheckMode::Normal
        }
    }

    pub fn graphql_ident(&self) -> Option<crate::Ident> {
        match (&self.rename, &self.ident) {
            (Some(rename), _) => Some(crate::Ident::for_field(&**rename).with_span(rename.span())),
            (_, Some(ident)) => Some(crate::Ident::from_proc_macro2(ident, None)),
            _ => None,
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
            data: darling::ast::Data::Struct(darling::ast::Fields::new(
                darling::ast::Style::Struct,
                vec![
                    FragmentDeriveField {
                        ident: Some(format_ident!("field_one")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false.into(),
                        recurse: None,
                        spread: false.into(),
                        rename: None,
                    },
                    FragmentDeriveField {
                        ident: Some(format_ident!("field_two")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: true.into(),
                        recurse: None,
                        spread: false.into(),
                        rename: None,
                    },
                    FragmentDeriveField {
                        ident: Some(format_ident!("field_three")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false.into(),
                        recurse: Some(8.into()),
                        spread: false.into(),
                        rename: Some("fieldThree".to_string().into()),
                    },
                    FragmentDeriveField {
                        ident: Some(format_ident!("some_spread")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false.into(),
                        recurse: None,
                        spread: true.into(),
                        rename: Some("fieldThree".to_string().into()),
                    },
                ],
            )),
            schema_path: "abcd".to_string().into(),
            query_module: None,
            schema_module_: None,
            graphql_type: Some("abcd".to_string().into()),
            argument_struct: None,
        };

        assert_matches!(input.validate(), Ok(()));
    }

    #[test]
    fn test_fragment_derive_validate_fails() {
        let input = FragmentDeriveInput {
            ident: format_ident!("TestInput"),
            data: darling::ast::Data::Struct(darling::ast::Fields::new(
                darling::ast::Style::Struct,
                vec![
                    FragmentDeriveField {
                        ident: Some(format_ident!("field_one")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false.into(),
                        recurse: None,
                        spread: false.into(),
                        rename: None,
                    },
                    FragmentDeriveField {
                        ident: Some(format_ident!("field_two")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: true.into(),
                        recurse: Some(8.into()),
                        spread: false.into(),
                        rename: None,
                    },
                    FragmentDeriveField {
                        ident: Some(format_ident!("field_three")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: true.into(),
                        recurse: Some(8.into()),
                        spread: false.into(),
                        rename: None,
                    },
                    FragmentDeriveField {
                        ident: Some(format_ident!("some_spread")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: true.into(),
                        recurse: None,
                        spread: true.into(),
                        rename: None,
                    },
                    FragmentDeriveField {
                        ident: Some(format_ident!("some_other_spread")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false.into(),
                        recurse: Some(8.into()),
                        spread: true.into(),
                        rename: None,
                    },
                ],
            )),
            schema_path: "abcd".to_string().into(),
            query_module: None,
            schema_module_: Some("abcd".to_string().into()),
            graphql_type: Some("abcd".to_string().into()),
            argument_struct: None,
        };

        let errors = input.validate().unwrap_err();
        assert_eq!(errors.len(), 4);
    }

    #[test]
    fn test_fragment_derive_validate_failed() {
        let input = FragmentDeriveInput {
            ident: format_ident!("TestInput"),
            data: darling::ast::Data::Struct(darling::ast::Fields::new(
                darling::ast::Style::Struct,
                vec![],
            )),
            schema_path: "abcd".to_string().into(),
            query_module: None,
            schema_module_: Some("abcd".to_string().into()),
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
            data: darling::ast::Data::Struct(darling::ast::Fields::new(
                darling::ast::Style::Struct,
                vec![
                    FragmentDeriveField {
                        ident: Some(format_ident!("field_one")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false.into(),
                        recurse: None,
                        spread: false.into(),
                        rename: None,
                    },
                    FragmentDeriveField {
                        ident: Some(format_ident!("field_two")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: true.into(),
                        recurse: None,
                        spread: false.into(),
                        rename: None,
                    },
                    FragmentDeriveField {
                        ident: Some(format_ident!("field_three")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false.into(),
                        recurse: Some(8.into()),
                        spread: false.into(),
                        rename: None,
                    },
                ],
            )),
            schema_path: "abcd".to_string().into(),
            query_module: None,
            schema_module_: Some("abcd".to_string().into()),
            graphql_type: None,
            argument_struct: None,
        };

        assert_matches!(input.validate(), Ok(()));
    }
}
