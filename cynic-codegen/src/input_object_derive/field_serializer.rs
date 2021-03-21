use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

use super::InputObjectDeriveField;
use crate::{
    schema::InputValue,
    type_validation::{check_types_are_compatible, CheckMode},
    FieldType, Ident, TypeIndex, TypePath,
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

    pub fn type_check_fn(&self, query_module_path: TypePath) -> TokenStream {
        // The check_types_are_compatible call in validate only checks for Option
        // and Vec wrappers - we don't have access to any info
        // about the types of fields within our current struct.
        //
        // So, we have to construct some functions with constraints
        // in order to make sure the fields are of the right type.

        let type_lock = self.graphql_field_type.as_type_lock(query_module_path);
        let wrapper_type = self.graphql_field_type.wrapper_path().unwrap();

        let rust_field_name = &self.rust_field.ident;
        let graphql_field_name = proc_macro2::Literal::string(&self.graphql_field.name);

        quote! {
            #[allow(clippy::ptr_arg)]
            fn #rust_field_name<SM: ::cynic::serde::ser::SerializeMap>(
                data: impl ::cynic::InputType<
                    #type_lock,
                    #wrapper_type
                >,
                map_serializer: &mut SM
            ) -> Result<(), SM::Error> {
                map_serializer.serialize_entry(
                    #graphql_field_name,
                    &data.into_serializable()
                )
            }
        }
    }

    pub fn field_insert_call(&self, serializer_ident: &proc_macro2::Ident) -> TokenStream {
        let field_span = self.rust_field.ident.span();
        let rust_field_name = &self.rust_field.ident;

        // For each field we just call our type checking function with the current field
        // and insert it into the output Map.
        let insert_call = quote_spanned! { field_span =>
            #rust_field_name(&self.#rust_field_name, &mut #serializer_ident)?;
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
