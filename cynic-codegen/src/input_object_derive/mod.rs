use proc_macro2::{Span, TokenStream};
use std::collections::HashSet;

use crate::{
    error::Errors,
    idents::RenameAll,
    load_schema,
    schema::{
        types::{InputObjectType, InputValue, TypeRef},
        Schema, Unvalidated,
    },
    suggestions::FieldSuggestionError,
    Ident,
};

mod field_serializer;
use field_serializer::FieldSerializer;

pub(crate) mod input;

#[cfg(test)]
mod tests;

use crate::suggestions::guess_field;
use input::InputObjectDeriveField;
pub use input::InputObjectDeriveInput;

pub fn input_object_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;

    let struct_span = ast.ident.span();

    match InputObjectDeriveInput::from_derive_input(ast) {
        Ok(input) => {
            let schema_doc = load_schema(&*input.schema_path)
                .map_err(|e| e.into_syn_error(input.schema_path.span()))?;

            let schema = Schema::new(&schema_doc);

            input_object_derive_impl(input, &schema, struct_span)
                .or_else(|e| Ok(e.to_compile_errors()))
        }
        Err(e) => Ok(e.write_errors()),
    }
}

pub fn input_object_derive_impl(
    input: InputObjectDeriveInput,
    schema: &Schema<'_, Unvalidated>,
    struct_span: Span,
) -> Result<TokenStream, Errors> {
    use quote::quote;

    let input_object = schema
        .lookup::<InputObjectType>(&input.graphql_type_name())
        .map_err(|e| syn::Error::new(input.graphql_type_span(), e))?;

    let rename_all = input.rename_all.unwrap_or(RenameAll::CamelCase);

    if let darling::ast::Data::Struct(fields) = &input.data {
        let ident = &input.ident;
        let input_marker_ident = Ident::for_type(&input.graphql_type_name());
        let schema_module = input.schema_module();

        let pairs = pair_fields(
            &fields.fields,
            &input_object,
            rename_all,
            input.require_all_fields,
            &struct_span,
        )?;

        let field_serializers = pairs
            .into_iter()
            .map(|(rust_field, graphql_field)| {
                FieldSerializer::new(rust_field, graphql_field, &schema_module)
            })
            .collect::<Vec<_>>();

        let errors = field_serializers
            .iter()
            .map(|fs| fs.validate())
            .flatten()
            .collect::<Errors>();

        if !errors.is_empty() {
            return Ok(errors.to_compile_errors());
        }

        let typechecks = field_serializers.iter().map(|fs| fs.type_check());
        let map_serializer_ident = proc_macro2::Ident::new("map_serializer", Span::call_site());
        let field_inserts = field_serializers
            .iter()
            .map(|fs| fs.field_insert_call(&map_serializer_ident));

        let map_len = field_serializers.len();

        let graphql_type_name = proc_macro2::Literal::string(input_object.name);

        Ok(quote! {
            #[automatically_derived]
            impl ::cynic::InputObject for #ident {
                type SchemaType = #schema_module::#input_marker_ident;
            }

            #[automatically_derived]
            impl ::cynic::serde::Serialize for #ident {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: ::cynic::serde::Serializer,
                {
                    use ::cynic::serde::ser::SerializeMap;
                    #(#typechecks)*

                    let mut map_serializer = serializer.serialize_map(Some(#map_len))?;

                    #(#field_inserts)*

                    map_serializer.end()
                }
            }

            ::cynic::impl_coercions!(#ident, #schema_module::#input_marker_ident);

            #[automatically_derived]
            impl #schema_module::variable::Variable for #ident {
                const TYPE: ::cynic::variables::VariableType = ::cynic::variables::VariableType::Named(#graphql_type_name);
            }
        })
    } else {
        Err(syn::Error::new(
            struct_span,
            "InputObject can only be derived on a struct".to_string(),
        )
        .into())
    }
}

fn pair_fields<'a>(
    fields: &'a [InputObjectDeriveField],
    input_object_def: &'a InputObjectType<'a>,
    rename_all: RenameAll,
    require_all_fields: bool,
    struct_span: &Span,
) -> Result<Vec<(&'a InputObjectDeriveField, &'a InputValue<'a>)>, Errors> {
    let mut result = Vec::new();
    let mut unknown_fields = Vec::new();

    for field in fields {
        let ident = field.graphql_ident(rename_all);
        match input_object_def.field(&ident) {
            Some(schema_field) => result.push((field, schema_field)),
            None => unknown_fields.push(field),
        }
    }

    let required_fields: HashSet<_>;
    if require_all_fields {
        required_fields = input_object_def.fields.iter().collect();
    } else {
        required_fields = input_object_def
            .fields
            .iter()
            .filter(|f| !matches!(f.value_type, TypeRef::Nullable(_)))
            .collect();
    }

    let provided_fields = result
        .iter()
        .map(|(_, field)| field)
        .copied()
        .collect::<HashSet<_>>();

    let missing_fields = required_fields
        .difference(&provided_fields)
        .collect::<Vec<_>>();

    if missing_fields.is_empty() && unknown_fields.is_empty() {
        return Ok(result);
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
                    graphql_type_name: input_object_def.name,
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

    use super::*;

    static SCHEMA: &str = r#"
        input TestType {
            field_one: String!,
            field_two: String
        }
        "#;

    #[test]
    fn test_join_fields_when_all_required() {
        let document = crate::schema::parse_schema(SCHEMA).unwrap();
        let schema = crate::schema::Schema::new(&document);
        let input_object = schema.lookup("TestType").unwrap();

        let fields = vec![InputObjectDeriveField {
            ident: Some(proc_macro2::Ident::new("field_one", Span::call_site())),
            ty: syn::parse_quote! { String },
            rename: None,
            skip_serializing_if: None,
        }];

        let result = pair_fields(
            &fields,
            &input_object,
            RenameAll::None,
            true,
            &Span::call_site(),
        );

        assert_matches!(result, Err(_))
    }

    #[test]
    fn test_join_fields_when_required_field_missing() {
        let document = crate::schema::parse_schema(SCHEMA).unwrap();
        let schema = crate::schema::Schema::new(&document);
        let input_object = schema.lookup("TestType").unwrap();

        let fields = vec![InputObjectDeriveField {
            ident: Some(proc_macro2::Ident::new("field_two", Span::call_site())),
            ty: syn::parse_quote! { String },
            rename: None,
            skip_serializing_if: None,
        }];

        let result = pair_fields(
            &fields,
            &input_object,
            RenameAll::None,
            false,
            &Span::call_site(),
        );

        assert_matches!(result, Err(_))
    }

    #[test]
    fn test_join_fields_when_not_required() {
        let document = crate::schema::parse_schema(SCHEMA).unwrap();
        let schema = crate::schema::Schema::new(&document);
        let input_object = schema.lookup("TestType").unwrap();

        let fields = vec![InputObjectDeriveField {
            ident: Some(proc_macro2::Ident::new("field_one", Span::call_site())),
            ty: syn::parse_quote! { String },
            rename: None,
            skip_serializing_if: None,
        }];

        let result = pair_fields(
            &fields,
            &input_object,
            RenameAll::None,
            false,
            &Span::call_site(),
        );

        assert_matches!(result, Ok(_));

        let (rust_field_ref, input_field_ref) = result.unwrap().into_iter().next().unwrap();
        assert!(std::ptr::eq(rust_field_ref, fields.first().unwrap()));
        assert!(std::ptr::eq(
            input_field_ref,
            input_object.fields.first().unwrap()
        ));
    }
}
