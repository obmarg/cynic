use crate::{
    input_object_derive::input::{OneOfDeriveField, OneOfDeriveVariant},
    schema::{Schema, Unvalidated},
};

use {
    proc_macro2::TokenStream,
    quote::{quote, quote_spanned},
};

use crate::{
    schema::types::{InputType, InputValue},
    types::{align_input_type, check_input_types_are_compatible},
};

pub struct FieldSerializer<'a> {
    rust_variant: &'a OneOfDeriveVariant,
    graphql_field: InputValue<'a>,
    schema_module: &'a syn::Path,
}

impl<'a> FieldSerializer<'a> {
    pub fn new(
        rust_variant: &'a OneOfDeriveVariant,
        graphql_field: InputValue<'a>,
        schema_module: &'a syn::Path,
    ) -> FieldSerializer<'a> {
        FieldSerializer {
            rust_variant,
            graphql_field,
            schema_module,
        }
    }

    fn rust_field(&self) -> &OneOfDeriveField {
        self.rust_variant
            .fields
            .fields
            .first()
            .expect("we only support newtype variants")
    }

    /// Validates the FieldSerializer definition, returning errors if there are
    /// any.
    pub fn validate(&self) -> Option<syn::Error> {
        // First, check for type errors
        if let Err(e) = check_input_types_are_compatible(&self.graphql_field, &self.rust_field().ty)
        {
            return Some(e);
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
            &self.rust_field().ty,
            &self.graphql_field.value_type,
            self.graphql_field.has_default,
        );

        quote! {
            cynic::assert_impl!(#aligned_type [#impl_generics] [#where_clause]: #trait_bound);
        }
    }

    pub fn field_match(&self, serializer_ident: &proc_macro2::Ident) -> TokenStream {
        let variant_span = self.rust_variant.ident.span();
        let rust_variant_name = &self.rust_variant.ident;
        let graphql_field_name = proc_macro2::Literal::string(self.graphql_field.name.as_str());

        quote_spanned! { variant_span =>
            Self::#rust_variant_name(contents) => {
                #serializer_ident.serialize_entry(
                    #graphql_field_name,
                    contents
                )?
            }
        }
    }
}
