use std::collections::HashSet;

use proc_macro2::Span;

use crate::{
    RenameAll,
    error::Errors,
    idents::RenamableFieldIdent,
    input_object_derive::input::{InputObjectDeriveField, OneOfDeriveVariant},
    schema::types::{InputObjectType, InputValue},
    suggestions::{FieldSuggestionError, guess_field},
};

pub(super) trait PairingCandidate {
    fn graphql_ident(&self, rename: RenameAll) -> RenamableFieldIdent;
}

impl PairingCandidate for InputObjectDeriveField {
    fn graphql_ident(&self, rename_rule: RenameAll) -> RenamableFieldIdent {
        InputObjectDeriveField::graphql_ident(self, rename_rule)
    }
}

impl PairingCandidate for OneOfDeriveVariant {
    fn graphql_ident(&self, rename_rule: RenameAll) -> RenamableFieldIdent {
        OneOfDeriveVariant::graphql_ident(self, rename_rule)
    }
}

pub(super) fn pair_fields<'a, T>(
    fields: &'a [T],
    input_object_def: InputObjectType<'a>,
    rename_all: RenameAll,
    require_all_fields: bool,
    struct_span: &Span,
) -> Result<Vec<(&'a T, InputValue<'a>)>, Errors>
where
    T: PairingCandidate,
{
    let mut result = Vec::new();
    let mut unknown_fields = Vec::new();

    for field in fields {
        let ident = field.graphql_ident(rename_all);
        match input_object_def.field(&ident) {
            Some(schema_field) => result.push((field, schema_field)),
            None => unknown_fields.push(field),
        }
    }

    let required_fields = if require_all_fields {
        input_object_def.fields.iter().collect::<HashSet<_>>()
    } else {
        input_object_def
            .fields
            .iter()
            .filter(|f| f.is_required())
            .collect::<HashSet<_>>()
    };

    let provided_fields = result
        .iter()
        .map(|(_, field)| field)
        .cloned()
        .collect::<HashSet<_>>();

    let missing_fields = required_fields
        .difference(&provided_fields)
        .collect::<Vec<_>>();

    if missing_fields.is_empty() && unknown_fields.is_empty() {
        return Ok(result.into_iter().map(|(l, r)| (l, r.clone())).collect());
    }

    let field_candidates = input_object_def
        .fields
        .iter()
        .map(|f| f.name.as_str())
        .collect::<Vec<_>>();

    let mut errors = unknown_fields
        .into_iter()
        .map(|field| {
            let field_name = &field.graphql_ident(rename_all);
            let graphql_name = field_name.graphql_name();
            let expected_field = graphql_name.as_str();
            let suggested_field = guess_field(field_candidates.iter().copied(), expected_field);
            syn::Error::new(
                field_name.span(),
                FieldSuggestionError {
                    expected_field,
                    graphql_type_name: input_object_def.name.as_ref(),
                    suggested_field,
                },
            )
        })
        .map(Errors::from)
        .collect::<Errors>();

    if !missing_fields.is_empty() {
        let missing_fields_string = missing_fields
            .into_iter()
            .map(|f| f.name.as_str().to_string())
            .collect::<Vec<_>>()
            .join(", ");

        errors.push(syn::Error::new(
            *struct_span,
            format!(
                "This InputObject is missing these fields: {}",
                missing_fields_string
            ),
        ))
    }

    Err(errors)
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;

    use crate::{
        input_object_derive::input::InputObjectDeriveField,
        schema::{Schema, SchemaInput},
    };

    use super::*;

    static SCHEMA: &str = r#"
        input TestType {
            field_one: String!,
            field_two: String
        }
        "#;

    #[test]
    fn test_join_fields_when_all_required() {
        let schema = Schema::new(SchemaInput::from_sdl(SCHEMA).unwrap());
        let input_object = schema.lookup("TestType").unwrap();

        let fields = vec![InputObjectDeriveField {
            ident: Some(proc_macro2::Ident::new("field_one", Span::call_site())),
            ty: syn::parse_quote! { String },
            rename: None,
            skip_serializing_if: None,
        }];

        let result = pair_fields(
            &fields,
            input_object,
            RenameAll::None,
            true,
            &Span::call_site(),
        );

        assert_matches!(result, Err(_))
    }

    #[test]
    fn test_join_fields_when_required_field_missing() {
        let schema = Schema::new(SchemaInput::from_sdl(SCHEMA).unwrap());
        let input_object = schema.lookup("TestType").unwrap();

        let fields = vec![InputObjectDeriveField {
            ident: Some(proc_macro2::Ident::new("field_two", Span::call_site())),
            ty: syn::parse_quote! { String },
            rename: None,
            skip_serializing_if: None,
        }];

        let result = pair_fields(
            &fields,
            input_object,
            RenameAll::None,
            false,
            &Span::call_site(),
        );

        assert_matches!(result, Err(_))
    }

    #[test]
    fn test_join_fields_when_not_required() {
        let schema = Schema::new(SchemaInput::from_sdl(SCHEMA).unwrap());
        let input_object = schema.lookup::<InputObjectType<'_>>("TestType").unwrap();

        let fields = vec![InputObjectDeriveField {
            ident: Some(proc_macro2::Ident::new("field_one", Span::call_site())),
            ty: syn::parse_quote! { String },
            rename: None,
            skip_serializing_if: None,
        }];

        let result = pair_fields(
            &fields,
            input_object.clone(),
            RenameAll::None,
            false,
            &Span::call_site(),
        );

        assert_matches!(result, Ok(_));

        let (rust_field_ref, input_field_ref) = result.unwrap().into_iter().next().unwrap();
        assert!(std::ptr::eq(rust_field_ref, fields.first().unwrap()));
        assert_eq!(&input_field_ref, input_object.fields.first().unwrap());
    }
}
