use crate::{
    input_object_derive::input::InputObjectDeriveField,
    schema::{Schema, Unvalidated},
};

use {
    proc_macro2::TokenStream,
    quote::{quote, quote_spanned},
    syn::spanned::Spanned,
};

use crate::{
    schema::types::{InputType, InputValue},
    types::{self, align_input_type, check_input_types_are_compatible},
};

pub struct FieldSerializer<'a> {
    rust_field: &'a InputObjectDeriveField,
    graphql_field: InputValue<'a>,
    schema_module: &'a syn::Path,
    parent_is_one_of: bool,
}

impl<'a> FieldSerializer<'a> {
    pub fn new(
        rust_field: &'a InputObjectDeriveField,
        graphql_field: InputValue<'a>,
        schema_module: &'a syn::Path,
        parent_is_one_of: bool,
    ) -> FieldSerializer<'a> {
        FieldSerializer {
            rust_field,
            graphql_field,
            schema_module,
            parent_is_one_of,
        }
    }

    /// Validates the FieldSerializer definition, returning errors if there are
    /// any.
    pub fn validate(&self) -> Option<syn::Error> {
        // First, check for type errors
        if let Err(e) = check_input_types_are_compatible(&self.graphql_field, &self.rust_field.ty) {
            return Some(e);
        }

        if self.rust_field.skip_serializing_if.is_some() && self.graphql_field.is_required() {
            return Some(syn::Error::new(
                self.rust_field.skip_serializing_if.as_ref().unwrap().span(),
                "You can't specify skip_serializing_if on a required field".to_string(),
            ));
        }

        None
    }

    pub fn type_check(
        &self,
        impl_generics: &syn::ImplGenerics<'_>,
        where_clause: Option<&syn::WhereClause>,
        schema: &Schema<'a, Unvalidated>,
    ) -> TokenStream {
        let marker_type = self
            .graphql_field
            .value_type
            .marker_type()
            .to_path(self.schema_module);

        let trait_bound = match self.graphql_field.value_type.inner_type(schema) {
            InputType::Scalar(_) => quote! { cynic::schema::IsScalar<#marker_type> },
            InputType::Enum(_) => quote! { cynic::Enum<SchemaType = #marker_type> },
            InputType::InputObject(_) => {
                quote! { cynic::InputObject<SchemaType = #marker_type> }
            }
        };

        let aligned_type = align_input_type(
            &self.rust_field.ty,
            &self.graphql_field.value_type,
            self.graphql_field.has_default,
        );

        quote! {
            cynic::assert_impl!(#aligned_type [#impl_generics] [#where_clause]: #trait_bound);
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

        match (
            &self.rust_field.skip_serializing_if,
            self.should_auto_skip_serializing(),
        ) {
            (Some(skip_check_fn), _) => {
                let skip_check_fn = &**skip_check_fn;
                quote! {
                    if !#skip_check_fn(&self.#rust_field_name) {
                        #insert_call
                    }
                }
            }
            (_, true) => {
                quote! {
                    if Option::is_some(&self.#rust_field_name) {
                        #insert_call
                    }
                }
            }
            _ => insert_call,
        }
    }

    fn should_auto_skip_serializing(&self) -> bool {
        types::outer_type_is_option(&self.rust_field.ty)
            && (self.parent_is_one_of
                || (self.graphql_field.has_default && !self.graphql_field.is_nullable()))
    }
}
