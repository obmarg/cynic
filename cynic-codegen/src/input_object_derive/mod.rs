use proc_macro2::{Span, TokenStream};
use std::collections::HashMap;
use syn::spanned::Spanned;

use crate::{
    ident::{RenameAll, RenameRule},
    load_schema,
    schema::{Definition, Document, InputObjectType, InputValue, TypeDefinition},
    Ident, TypeIndex,
};

mod field_serializer;
use field_serializer::FieldSerializer;

pub(crate) mod input;

use crate::suggestions::{format_guess, guess_field};
use input::InputObjectDeriveField;
pub use input::InputObjectDeriveInput;

pub fn input_object_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;

    let enum_span = ast.span();

    match InputObjectDeriveInput::from_derive_input(ast) {
        Ok(input) => load_schema(&*input.schema_path)
            .map_err(|e| e.into_syn_error(input.schema_path.span()))
            .and_then(|schema| input_object_derive_impl(input, &schema, enum_span))
            .or_else(|e| Ok(e.to_compile_error())),
        Err(e) => Ok(e.write_errors()),
    }
}

pub fn input_object_derive_impl(
    input: InputObjectDeriveInput,
    schema: &Document,
    struct_span: Span,
) -> Result<TokenStream, syn::Error> {
    use quote::quote;

    let input_object_def = schema.definitions.iter().find_map(|def| {
        if let Definition::TypeDefinition(TypeDefinition::InputObject(obj)) = def {
            if obj.name == *input.graphql_type {
                return Some(obj);
            }
        }
        None
    });
    if input_object_def.is_none() {
        let candidates = schema.definitions.iter().flat_map(|def| {
            if let Definition::TypeDefinition(TypeDefinition::InputObject(obj)) = def {
                Some(obj.name.as_str())
            } else {
                None
            }
        });
        let guess_field = guess_field(candidates, &(*(input.graphql_type)));
        return Err(syn::Error::new(
            input.graphql_type.span(),
            format!(
                "Could not find an InputObject named {} in {}.{}",
                *input.graphql_type,
                *input.schema_path,
                format_guess(guess_field)
            ),
        ));
    }
    let input_object_def = input_object_def.unwrap();

    let type_index = TypeIndex::for_schema(&schema);

    let rename_all = input.rename_all.unwrap_or(RenameAll::CamelCase);

    if let darling::ast::Data::Struct(fields) = &input.data {
        let ident = &input.ident;
        let input_marker_ident = Ident::for_type(&*input.graphql_type);
        let query_module = Ident::for_module(&input.query_module);
        let input_object_name = ident.to_string();

        let pairs = match join_fields(
            &fields.fields,
            input_object_def,
            &input_object_name,
            rename_all,
            input.require_all_fields,
            &struct_span,
        ) {
            Ok(pairs) => pairs,
            Err(error_tokens) => return Ok(error_tokens),
        };

        let field_serializers = pairs
            .into_iter()
            .map(|(rust_field, graphql_field)| {
                FieldSerializer::new(rust_field, graphql_field, &type_index, &query_module)
            })
            .collect::<Vec<_>>();

        let errors = field_serializers
            .iter()
            .map(|fs| fs.validate())
            .flatten()
            .collect::<Vec<_>>();

        if !errors.is_empty() {
            return Ok(errors.into_iter().map(|e| e.to_compile_error()).fold(
                TokenStream::new(),
                |mut a, b| {
                    a.extend(b);
                    a
                },
            ));
        }

        let output_struct = proc_macro2::Ident::new("output", Span::call_site());
        let typecheck_funcs = field_serializers.iter().map(|fs| fs.type_check_fn());
        let field_inserts = field_serializers
            .iter()
            .map(|fs| fs.field_insert_call(&output_struct));

        let map_len = field_serializers.len();

        Ok(quote! {
            #[automatically_derived]
            impl ::cynic::InputObject<#query_module::#input_marker_ident> for #ident {}

            #[automatically_derived]
            impl ::cynic::SerializableArgument for #ident {
                fn serialize(&self) -> Result<::cynic::serde_json::Value, ::cynic::SerializeError> {
                    use ::cynic::{Scalar, Enum, SerializableArgument};
                    #(
                        #typecheck_funcs
                    )*

                    let mut output = ::cynic::serde_json::map::Map::with_capacity(#map_len);

                    #(#field_inserts)*

                    Ok(::cynic::serde_json::Value::Object(output))
                }
            }

            ::cynic::impl_into_argument_for_options!(#ident);
        })
    } else {
        Err(syn::Error::new(
            struct_span,
            "Enum can only be derived from an enum".to_string(),
        ))
    }
}

fn join_fields<'a>(
    fields: &'a [InputObjectDeriveField],
    input_object_def: &'a InputObjectType,
    input_object_name: &str,
    rename_all: RenameAll,
    require_all_fields: bool,
    struct_span: &Span,
) -> Result<Vec<(&'a InputObjectDeriveField, &'a InputValue)>, TokenStream> {
    use crate::schema::TypeExt;

    let mut map = HashMap::new();
    for field in fields {
        let transformed_ident = Ident::from_proc_macro2(
            field
                .ident
                .as_ref()
                .expect("InputObject derive was passed a tuple struct or similar"),
            RenameRule::new(rename_all, field.rename.as_ref()),
        );
        map.insert(transformed_ident, (Some(field), None));
    }

    for value in &input_object_def.fields {
        let mut entry = map.entry(Ident::new(&value.name)).or_insert((None, None));
        entry.1 = Some(value);
    }

    let mut missing_required_fields = vec![];
    let mut missing_optional_fields = vec![];
    let mut errors = TokenStream::new();
    for (transformed_ident, value) in map.iter() {
        match value {
            (None, Some(input_value)) if input_value.value_type.is_required() => {
                missing_required_fields.push(input_value.name.as_ref())
            }
            (None, Some(input_value)) => missing_optional_fields.push(input_value.name.as_ref()),
            (Some(field), None) => {
                let candidates = map.values().flat_map(|v| match v.1 {
                    Some(input) => Some(input.name.as_str()),
                    None => None,
                });
                let guess_field = guess_field(candidates, &(transformed_ident.graphql_name()));
                errors.extend(
                    syn::Error::new(
                        field.ident.span(),
                        format!(
                            "Could not find a field {} in the GraphQL InputObject {}.{}",
                            transformed_ident.graphql_name(),
                            input_object_name,
                            format_guess(guess_field)
                        ),
                    )
                    .to_compile_error(),
                )
            }
            _ => (),
        }
    }

    if !missing_required_fields.is_empty() {
        let missing_fields_string = missing_required_fields.join(", ");
        errors.extend(
            syn::Error::new(
                *struct_span,
                format!(
                    "This InputObject is missing these required fields: {}",
                    missing_fields_string
                ),
            )
            .to_compile_error(),
        )
    }

    if require_all_fields && !missing_optional_fields.is_empty() {
        let missing_fields_string = missing_optional_fields.join(", ");
        errors.extend(
            syn::Error::new(
                *struct_span,
                format!(
                    "This InputObject is missing these optional fields: {}.  If you wish to omit them you can remove the `require_all_fields` attribute from the InputObject",
                    missing_fields_string
                ),
            )
            .to_compile_error(),
        )
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(map
        .into_iter()
        .filter(|(_, (rust_field, _))| rust_field.is_some())
        .map(|(_, (a, b))| (a.unwrap(), b.unwrap()))
        .collect())
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;

    use super::*;
    use crate::schema::{Definition, InputObjectType, TypeDefinition};

    fn test_input_object() -> InputObjectType {
        let schema = r#"
        input TestType {
            field_one: String!,
            field_two: String
        }
        "#;

        if let Definition::TypeDefinition(TypeDefinition::InputObject(rv)) =
            crate::schema::parse_schema(schema)
                .unwrap()
                .definitions
                .into_iter()
                .next()
                .unwrap()
        {
            rv
        } else {
            panic!("Parsing failed")
        }
    }

    #[test]
    fn test_join_fields_when_all_required() {
        let fields = vec![InputObjectDeriveField {
            ident: Some(proc_macro2::Ident::new("field_one", Span::call_site())),
            ty: syn::parse_quote! { String },
            rename: None,
            skip_serializing_if: None,
        }];
        let input_object = test_input_object();

        let result = join_fields(
            &fields,
            &input_object,
            "MyInputObject",
            RenameAll::None,
            true,
            &Span::call_site(),
        );

        assert_matches!(result, Err(_))
    }

    #[test]
    fn test_join_fields_when_required_field_missing() {
        let fields = vec![InputObjectDeriveField {
            ident: Some(proc_macro2::Ident::new("field_two", Span::call_site())),
            ty: syn::parse_quote! { String },
            rename: None,
            skip_serializing_if: None,
        }];
        let input_object = test_input_object();

        let result = join_fields(
            &fields,
            &input_object,
            "MyInputObject",
            RenameAll::None,
            false,
            &Span::call_site(),
        );

        assert_matches!(result, Err(_))
    }

    #[test]
    fn test_join_fields_when_not_required() {
        let fields = vec![InputObjectDeriveField {
            ident: Some(proc_macro2::Ident::new("field_one", Span::call_site())),
            ty: syn::parse_quote! { String },
            rename: None,
            skip_serializing_if: None,
        }];
        let input_object = test_input_object();

        let result = join_fields(
            &fields,
            &input_object,
            "MyInputObject",
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
