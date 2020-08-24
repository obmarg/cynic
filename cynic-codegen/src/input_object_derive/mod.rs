use proc_macro2::{Span, TokenStream};
use std::collections::HashMap;
use syn::spanned::Spanned;

use crate::{
    ident::{RenameAll, RenameRule},
    load_schema,
    schema::{Definition, Document, InputObjectType, InputValue, TypeDefinition},
    type_validation::check_types_are_compatible,
    FieldType, Ident, TypeIndex,
};

pub(crate) mod input;

use input::InputObjectDeriveField;
pub use input::InputObjectDeriveInput;

pub fn input_object_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;

    let enum_span = ast.span();

    match InputObjectDeriveInput::from_derive_input(ast) {
        Ok(input) => load_schema(&*input.schema_path)
            .map_err(|e| e.to_syn_error(input.schema_path.span()))
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
    use quote::{quote, quote_spanned};

    let input_object_def = schema.definitions.iter().find_map(|def| {
        if let Definition::TypeDefinition(TypeDefinition::InputObject(obj)) = def {
            if obj.name == *input.graphql_type {
                return Some(obj);
            }
        }
        None
    });
    if input_object_def.is_none() {
        return Err(syn::Error::new(
            input.graphql_type.span(),
            format!(
                "Could not find an InputObject named {} in {}",
                *input.graphql_type, *input.schema_path
            ),
        ));
    }
    let input_object_def = input_object_def.unwrap();

    let type_index = TypeIndex::for_schema(&schema);

    if let darling::ast::Data::Struct(fields) = &input.data {
        let pairs = match join_fields(
            &fields.fields,
            input_object_def,
            &input.ident.to_string(),
            input.rename_all,
            &struct_span,
        ) {
            Ok(pairs) => pairs,
            Err(error_tokens) => return Ok(error_tokens),
        };

        // Check the types of our struct align with the GQL types.
        let mut type_errors = Vec::new();
        for (rust_field, gql_value) in pairs.iter() {
            let field_type = crate::FieldType::from_schema_type(&gql_value.value_type, &type_index);
            if let Err(e) = check_types_are_compatible(&field_type, &rust_field.ty, false) {
                type_errors.push(e);
            }
        }
        if !type_errors.is_empty() {
            return Ok(type_errors.into_iter().map(|e| e.to_compile_error()).fold(
                TokenStream::new(),
                |mut a, b| {
                    a.extend(b);
                    a
                },
            ));
        }

        let ident = input.ident;
        let query_module = Ident::for_module(&input.query_module);
        let input_marker_ident = Ident::for_type(&*input.graphql_type);

        let gql_field_names = pairs
            .iter()
            .map(|(_, gql_value)| proc_macro2::Literal::string(&gql_value.name));

        let typecheck_funcs = pairs.iter().map(|(rust_field, gql_value)| {
            // The check_types_are_compatible call above only checks for Option
            // and Vec wrappers - we don't have access to any of the type
            // information of fields within our current struct.
            //
            // So, we have to construct some functions with constraints
            // in order to make sure the fields are of the right type.
            //
            // That's what this block is doing.

            let field_type = FieldType::from_schema_type(&gql_value.value_type, &type_index);
            let generic_param = field_type.generic_parameter(Ident::new("T"));
            let arg_type = field_type.to_tokens(
                generic_param.as_ref().map(|p| p.name.clone()),
                query_module.clone().into(),
            );

            let rust_field_name = &rust_field.ident;
            let generic_param_definition =
                generic_param.map(|p| p.to_tokens(query_module.clone().into()));

            quote! {
                fn #rust_field_name<#generic_param_definition>(data: &#arg_type) ->
                    Result<::cynic::serde_json::Value, ::cynic::SerializeError> {
                        data.serialize()
                    }
            }
        });

        let field_values = pairs.iter().map(|(rust_field, _)| {
            let field_span = rust_field.ident.span();
            let rust_field_name = &rust_field.ident;

            // For each field we just call our type checking function with the current field
            quote_spanned! { field_span =>
                #rust_field_name(&self.#rust_field_name)?
            }
        });

        Ok(quote! {
            #[automatically_derived]
            impl ::cynic::InputObject<#query_module::#input_marker_ident> for #ident {
                fn serialize(&self) -> Result<::cynic::serde_json::Value, ::cynic::SerializeError>  {
                    use ::cynic::{Scalar, Enum, SerializableArgument};
                    #(
                        #typecheck_funcs
                    )*

                    Ok(::cynic::serde_json::json!({
                        #(
                            #gql_field_names: #field_values,
                        )*
                    }))
                }
            }

            #[automatically_derived]
            impl ::cynic::SerializableArgument for #ident {
                fn serialize(&self) -> Result<::cynic::serde_json::Value, ::cynic::SerializeError> {
                    self.serialize()
                }
            }

            // TODO: Figure out if this does what I want...
            ::cynic::define_into_argument_for_scalar!(#ident);
        })
    } else {
        Err(syn::Error::new(
            struct_span,
            format!("Enum can only be derived from an enum"),
        ))
    }
}

fn join_fields<'a>(
    fields: &'a [InputObjectDeriveField],
    input_object_def: &'a InputObjectType,
    input_object_name: &str,
    rename_all: Option<RenameAll>,
    struct_span: &Span,
) -> Result<Vec<(&'a InputObjectDeriveField, &'a InputValue)>, TokenStream> {
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

    let mut missing_fields = vec![];
    let mut errors = TokenStream::new();
    for (transformed_ident, value) in map.iter() {
        match value {
            (None, Some(input_value)) => missing_fields.push(input_value.name.as_ref()),
            (Some(field), None) => errors.extend(
                syn::Error::new(
                    field.ident.span(),
                    format!(
                        "Could not find a field {} in the GraphQL InputObject {}",
                        transformed_ident, input_object_name
                    ),
                )
                .to_compile_error(),
            ),
            _ => (),
        }
    }
    if !missing_fields.is_empty() {
        let missing_fields_string = missing_fields.join(", ");
        errors.extend(
            syn::Error::new(
                struct_span.clone(),
                format!("Missing fields: {}", missing_fields_string),
            )
            .to_compile_error(),
        )
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(map
        .into_iter()
        .map(|(_, (a, b))| (a.unwrap(), b.unwrap()))
        .collect())
}
