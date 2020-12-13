use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

use super::InputObjectDeriveField;
use crate::{
    schema::InputValue,
    type_validation::{check_types_are_compatible, CheckMode},
    FieldType, Ident, TypeIndex,
};

pub struct FieldSerializer<'a> {
    rust_field: &'a InputObjectDeriveField,
    graphql_field: &'a InputValue,
    graphql_field_type: FieldType,
    query_module: &'a Ident,
}

impl<'a> FieldSerializer<'a> {
    pub fn new(
        rust_field: &'a InputObjectDeriveField,
        graphql_field: &'a InputValue,
        type_index: &TypeIndex,
        query_module: &'a Ident,
    ) -> FieldSerializer<'a> {
        FieldSerializer {
            rust_field,
            graphql_field,
            graphql_field_type: FieldType::from_schema_type(&graphql_field.value_type, &type_index),
            query_module,
        }
    }

    /// Validates the FieldSerializer definition, returning errors if there are any.
    pub fn validate(&self) -> Option<syn::Error> {
        // First, check for type errors
        if let Err(e) = check_types_are_compatible(
            &self.graphql_field_type,
            &self.rust_field.ty,
            CheckMode::Normal,
        ) {
            return Some(e);
        }

        if self.rust_field.skip_serializing_if.is_some() && !self.graphql_field_type.is_nullable() {
            return Some(syn::Error::new(
                self.rust_field.skip_serializing_if.as_ref().unwrap().span(),
                "You can't specify skip_serializing_if on a required field".to_string(),
            ));
        }

        None
    }

    pub fn type_check_fn(&self) -> TokenStream {
        // The check_types_are_compatible call in validate only checks for Option
        // and Vec wrappers - we don't have access to any info
        // about the types of fields within our current struct.
        //
        // So, we have to construct some functions with constraints
        // in order to make sure the fields are of the right type.

        let generic_param = self.graphql_field_type.generic_parameter(Ident::new("T"));
        let arg_type = self.graphql_field_type.to_tokens(
            generic_param.as_ref().map(|p| p.name.clone()),
            self.query_module.clone().into(),
        );

        let rust_field_name = &self.rust_field.ident;
        let generic_param_definition =
            generic_param.map(|p| p.to_tokens(self.query_module.clone().into()));

        quote! {
            #[allow(clippy::ptr_arg)]
            fn #rust_field_name<#generic_param_definition>(data: &#arg_type) ->
                Result<::cynic::serde_json::Value, ::cynic::SerializeError> {
                    data.serialize()
                }
        }
    }

    pub fn field_insert_call(&self, output_struct: &proc_macro2::Ident) -> TokenStream {
        let field_span = self.rust_field.ident.span();
        let rust_field_name = &self.rust_field.ident;

        let gql_field_name = proc_macro2::Literal::string(&self.graphql_field.name);

        // For each field we just call our type checking function with the current field
        // and insert it into the output Map.
        let insert_call = quote_spanned! { field_span =>
            #output_struct.insert(#gql_field_name.to_string(), #rust_field_name(&self.#rust_field_name)?);
        };

        if let Some(skip_check_fn) = &self.rust_field.skip_serializing_if {
            let skip_check_fn = &**skip_check_fn;
            quote! {
                if !#skip_check_fn(&self.#rust_field_name) {
                    #insert_call
                }
            }
        } else {
            insert_call
        }
    }
}
