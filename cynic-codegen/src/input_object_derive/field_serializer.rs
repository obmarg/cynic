use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

use super::InputObjectDeriveField;
use crate::{
    schema::types::{InputType, InputValue, TypeRef},
    type_validation::{check_types_are_compatible, CheckMode},
};

pub struct FieldSerializer<'a> {
    rust_field: &'a InputObjectDeriveField,
    graphql_field: &'a InputValue<'a>,
    schema_module: &'a syn::Path,
}

impl<'a> FieldSerializer<'a> {
    pub fn new(
        rust_field: &'a InputObjectDeriveField,
        graphql_field: &'a InputValue,
        schema_module: &'a syn::Path,
    ) -> FieldSerializer<'a> {
        FieldSerializer {
            rust_field,
            graphql_field,
            schema_module,
        }
    }

    /// Validates the FieldSerializer definition, returning errors if there are any.
    pub fn validate(&self) -> Option<syn::Error> {
        // First, check for type errors
        if let Err(e) = check_types_are_compatible(
            &self.graphql_field.value_type,
            &self.rust_field.ty,
            CheckMode::InputTypes,
        ) {
            return Some(e);
        }

        let nullable = matches!(self.graphql_field.value_type, TypeRef::Nullable(_));

        if self.rust_field.skip_serializing_if.is_some() && !nullable {
            return Some(syn::Error::new(
                self.rust_field.skip_serializing_if.as_ref().unwrap().span(),
                "You can't specify skip_serializing_if on a non nullable field".to_string(),
            ));
        }

        None
    }

    pub fn type_check(&self) -> TokenStream {
        let ty = &self.rust_field.ty;
        let marker_type = self
            .graphql_field
            .value_type
            .marker_type()
            .to_path(self.schema_module);

        let trait_bound = match self.graphql_field.value_type.inner_type() {
            InputType::Scalar(_) => quote! { ::cynic::schema::IsScalar<#marker_type> },
            InputType::Enum(_) => quote! { ::cynic::Enum<SchemaType = #marker_type> },
            InputType::InputObject(_) => {
                quote! { ::cynic::InputObject<SchemaType = #marker_type> }
            }
        };

        quote! {
            ::cynic::assert_impl!(#ty: #trait_bound);
        }
    }

    pub fn field_insert_call(&self, serializer_ident: &proc_macro2::Ident) -> TokenStream {
        let field_span = self.rust_field.ident.span();
        let rust_field_name = &self.rust_field.ident;
        let graphql_field_name = proc_macro2::Literal::string(self.graphql_field.name.as_str());

        let insert_call = quote_spanned! { field_span =>
            #serializer_ident.serialize_entry(
                #graphql_field_name,
                &self.#rust_field_name
            )?;
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
