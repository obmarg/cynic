use super::directives::FieldDirective;

use {darling::util::SpannedValue, proc_macro2::Span, std::collections::HashSet};

use crate::{idents::RenamableFieldIdent, schema::SchemaInput, types::CheckMode, Errors};

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(struct_named))]
pub struct FragmentDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) data: darling::ast::Data<(), RawFragmentDeriveField>,
    pub(super) generics: syn::Generics,

    #[darling(default)]
    schema: Option<SpannedValue<String>>,
    #[darling(default)]
    schema_path: Option<SpannedValue<String>>,

    #[darling(default, rename = "schema_module")]
    schema_module_: Option<syn::Path>,

    #[darling(default)]
    pub graphql_type: Option<SpannedValue<String>>,

    #[darling(default)]
    pub(super) no_deserialize: bool,

    #[darling(default)]
    variables: Option<syn::Path>,
}

impl FragmentDeriveInput {
    pub fn schema_module(&self) -> syn::Path {
        if let Some(schema_module) = &self.schema_module_ {
            return schema_module.clone();
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

    pub fn validate(&self) -> Result<Vec<FragmentDeriveField>, Errors> {
        let data_field_is_empty = matches!(self.data.clone(), darling::ast::Data::Struct(fields) if fields.fields.is_empty());
        if data_field_is_empty {
            return Err(syn::Error::new(
                self.ident.span(),
                format!(
                    "At least one field should be selected for `{}`.",
                    self.ident
                ),
            )
            .into());
        }

        let mut fields = vec![];
        let mut errors = Errors::default();

        let results = self
            .data
            .clone()
            .map_struct_fields(|field| field.validate())
            .take_struct()
            .unwrap()
            .into_iter();

        for result in results {
            match result {
                Ok(field) => fields.push(field),
                Err(error) => errors.extend(error),
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(fields)
    }

    pub fn detect_aliases(&mut self) {
        let mut names = HashSet::new();
        if let darling::ast::Data::Struct(fields) = &mut self.data {
            for field in &mut fields.fields {
                if let Some(rename) = &mut field.rename {
                    let name = rename.as_str();
                    if names.contains(name) {
                        field.alias = true.into();
                        continue;
                    }
                    names.insert(name);
                }
            }
        }
    }

    pub fn variables(&self) -> Option<syn::Path> {
        self.variables.clone()
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
}

#[derive(darling::FromField, Clone)]
#[darling(attributes(cynic), forward_attrs(arguments, directives))]
pub struct RawFragmentDeriveField {
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

    #[darling(default)]
    alias: SpannedValue<bool>,

    #[darling(default)]
    pub(super) feature: Option<SpannedValue<String>>,

    #[darling(default)]
    pub(super) default: SpannedValue<bool>,
}

pub struct FragmentDeriveField {
    pub(super) raw_field: RawFragmentDeriveField,

    pub(super) directives: Vec<super::directives::FieldDirective>,
}

impl RawFragmentDeriveField {
    pub fn validate(self) -> Result<FragmentDeriveField, Errors> {
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

        if *self.alias && self.rename.is_none() {
            return Err(syn::Error::new(
                self.alias.span(),
                "You can only alias a renamed field.  Try removing `alias` or adding a rename",
            )
            .into());
        }

        if *self.default && *self.spread {
            return Err(syn::Error::new(
                self.default.span(),
                "A field can't be defaulted if it's also being spread",
            )
            .into());
        }

        if *self.default && self.recurse.is_some() {
            return Err(syn::Error::new(
                self.recurse.unwrap().span(),
                "A field can't be recurse if it's also being defaulted",
            )
            .into());
        }

        if *self.default && *self.flatten {
            return Err(syn::Error::new(
                self.default.span(),
                "A field can't be defaulted if it's being flattened",
            )
            .into());
        }

        let directives = super::directives::directives_from_field_attrs(&self.attrs)?;
        let skippable = directives.iter().any(|directive| {
            matches!(
                directive,
                FieldDirective::Include(_) | FieldDirective::Skip(_)
            )
        });

        if skippable {
            if *self.spread {
                return Err(syn::Error::new(
                    self.spread.span(),
                    "spread can't currently be used on fields with skip or include directives",
                )
                .into());
            } else if *self.flatten {
                return Err(syn::Error::new(
                    self.flatten.span(),
                    "flatten can't currently be used on fields with skip or include directives",
                )
                .into());
            } else if let Some(recurse) = self.recurse {
                return Err(syn::Error::new(
                    recurse.span(),
                    "recurse can't currently be used on fields with skip or include directives",
                )
                .into());
            }
        }

        Ok(FragmentDeriveField {
            directives,
            raw_field: self,
        })
    }
}

impl FragmentDeriveField {
    pub(super) fn type_check_mode(&self) -> CheckMode {
        if *self.raw_field.flatten {
            CheckMode::Flattening
        } else if self.raw_field.recurse.is_some() {
            CheckMode::Recursing
        } else if *self.raw_field.spread {
            CheckMode::Spreading
        } else if self.has_default() {
            CheckMode::Defaulted
        } else if self.is_skippable() {
            CheckMode::Skippable
        } else {
            CheckMode::OutputTypes
        }
    }

    pub(super) fn is_skippable(&self) -> bool {
        self.directives.iter().any(|directive| {
            matches!(
                directive,
                FieldDirective::Include(_) | FieldDirective::Skip(_)
            )
        })
    }

    pub(super) fn spread(&self) -> bool {
        *self.raw_field.spread
    }

    pub(super) fn ident(&self) -> Option<&proc_macro2::Ident> {
        self.raw_field.ident.as_ref()
    }

    pub(super) fn graphql_ident(&self) -> RenamableFieldIdent {
        let mut ident = RenamableFieldIdent::from(
            self.raw_field
                .ident
                .clone()
                .expect("FragmentDerive only supports named structs"),
        );
        if let Some(rename) = &self.raw_field.rename {
            let span = rename.span();
            let rename = (**rename).clone();
            ident.set_rename(rename, span)
        }
        ident
    }

    pub(super) fn alias(&self) -> Option<String> {
        self.raw_field.alias.then(|| {
            self.raw_field
                .ident
                .as_ref()
                .expect("ident is required")
                .to_string()
        })
    }

    pub(super) fn has_default(&self) -> bool {
        *self.raw_field.default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use quote::format_ident;

    #[test]
    fn test_fragment_derive_validate_pass() {
        let input = FragmentDeriveInput {
            ident: format_ident!("TestInput"),
            data: darling::ast::Data::Struct(darling::ast::Fields::new(
                darling::ast::Style::Struct,
                vec![
                    RawFragmentDeriveField {
                        ident: Some(format_ident!("field_one")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false.into(),
                        recurse: None,
                        spread: false.into(),
                        rename: None,
                        alias: false.into(),
                        feature: None,
                        default: false.into(),
                    },
                    RawFragmentDeriveField {
                        ident: Some(format_ident!("field_two")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: true.into(),
                        recurse: None,
                        spread: false.into(),
                        rename: None,
                        alias: false.into(),
                        feature: None,
                        default: false.into(),
                    },
                    RawFragmentDeriveField {
                        ident: Some(format_ident!("field_three")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false.into(),
                        recurse: Some(8.into()),
                        spread: false.into(),
                        rename: Some("fieldThree".to_string().into()),
                        alias: false.into(),
                        feature: None,
                        default: false.into(),
                    },
                    RawFragmentDeriveField {
                        ident: Some(format_ident!("some_spread")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false.into(),
                        recurse: None,
                        spread: true.into(),
                        rename: Some("fieldThree".to_string().into()),
                        alias: true.into(),
                        feature: None,
                        default: false.into(),
                    },
                ],
            )),
            generics: Default::default(),
            schema: None,
            schema_path: Some("abcd".to_string().into()),
            schema_module_: None,
            graphql_type: Some("abcd".to_string().into()),
            variables: None,
            no_deserialize: false,
        };

        assert!(input.validate().is_ok());
    }

    #[test]
    fn test_fragment_derive_validate_fails() {
        let input = FragmentDeriveInput {
            ident: format_ident!("TestInput"),
            data: darling::ast::Data::Struct(darling::ast::Fields::new(
                darling::ast::Style::Struct,
                vec![
                    RawFragmentDeriveField {
                        ident: Some(format_ident!("field_one")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false.into(),
                        recurse: None,
                        spread: false.into(),
                        rename: None,
                        alias: false.into(),
                        feature: None,
                        default: false.into(),
                    },
                    RawFragmentDeriveField {
                        ident: Some(format_ident!("field_two")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: true.into(),
                        recurse: Some(8.into()),
                        spread: false.into(),
                        rename: None,
                        alias: false.into(),
                        feature: None,
                        default: false.into(),
                    },
                    RawFragmentDeriveField {
                        ident: Some(format_ident!("field_three")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: true.into(),
                        recurse: Some(8.into()),
                        spread: false.into(),
                        rename: None,
                        alias: false.into(),
                        feature: None,
                        default: false.into(),
                    },
                    RawFragmentDeriveField {
                        ident: Some(format_ident!("some_spread")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: true.into(),
                        recurse: None,
                        spread: true.into(),
                        rename: None,
                        alias: false.into(),
                        feature: None,
                        default: false.into(),
                    },
                    RawFragmentDeriveField {
                        ident: Some(format_ident!("some_other_spread")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false.into(),
                        recurse: Some(8.into()),
                        spread: true.into(),
                        rename: None,
                        alias: false.into(),
                        feature: None,
                        default: false.into(),
                    },
                    RawFragmentDeriveField {
                        ident: Some(format_ident!("some_other_spread")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false.into(),
                        recurse: Some(8.into()),
                        spread: true.into(),
                        rename: None,
                        alias: true.into(),
                        feature: None,
                        default: false.into(),
                    },
                ],
            )),
            generics: Default::default(),
            schema: None,
            schema_path: Some("abcd".to_string().into()),
            schema_module_: Some(syn::parse2(quote::quote! { abcd }).unwrap()),
            graphql_type: Some("abcd".to_string().into()),
            variables: None,
            no_deserialize: false,
        };

        let errors = input.validate().map(|_| ()).unwrap_err();
        assert_eq!(errors.len(), 5);
    }

    #[test]
    fn test_fragment_derive_validate_failed() {
        let input = FragmentDeriveInput {
            ident: format_ident!("TestInput"),
            data: darling::ast::Data::Struct(darling::ast::Fields::new(
                darling::ast::Style::Struct,
                vec![],
            )),
            generics: Default::default(),
            schema: None,
            schema_path: Some("abcd".to_string().into()),
            schema_module_: Some(syn::parse2(quote::quote! { abcd }).unwrap()),
            graphql_type: Some("abcd".to_string().into()),
            variables: None,
            no_deserialize: false,
        };
        let errors = input.validate().map(|_| ()).unwrap_err();
        insta::assert_snapshot!(errors.to_compile_errors().to_string(), @r###":: core :: compile_error ! { "At least one field should be selected for `TestInput`." }"###);
    }

    #[test]
    fn test_fragment_derive_validate_pass_no_graphql_type() {
        let input = FragmentDeriveInput {
            ident: format_ident!("TestInput"),
            data: darling::ast::Data::Struct(darling::ast::Fields::new(
                darling::ast::Style::Struct,
                vec![
                    RawFragmentDeriveField {
                        ident: Some(format_ident!("field_one")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false.into(),
                        recurse: None,
                        spread: false.into(),
                        rename: None,
                        alias: false.into(),
                        feature: None,
                        default: false.into(),
                    },
                    RawFragmentDeriveField {
                        ident: Some(format_ident!("field_two")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: true.into(),
                        recurse: None,
                        spread: false.into(),
                        rename: None,
                        alias: false.into(),
                        feature: None,
                        default: false.into(),
                    },
                    RawFragmentDeriveField {
                        ident: Some(format_ident!("field_three")),
                        ty: syn::parse_quote! { String },
                        attrs: vec![],
                        flatten: false.into(),
                        recurse: Some(8.into()),
                        spread: false.into(),
                        rename: None,
                        alias: false.into(),
                        feature: None,
                        default: false.into(),
                    },
                ],
            )),
            generics: Default::default(),
            schema: None,
            schema_path: Some("abcd".to_string().into()),
            schema_module_: Some(syn::parse2(quote::quote! { abcd }).unwrap()),
            graphql_type: None,
            variables: None,
            no_deserialize: false,
        };

        assert!(input.validate().is_ok())
    }
}
