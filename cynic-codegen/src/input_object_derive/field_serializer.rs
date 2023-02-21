use {
    proc_macro2::TokenStream,
    quote::{format_ident, quote, quote_spanned},
    syn::{
        spanned::Spanned,
        visit_mut::{self, VisitMut},
    },
};

use {
    super::InputObjectDeriveField,
    crate::{
        schema::types::{InputType, InputValue},
        types::{self, align_input_type, check_input_types_are_compatible},
    },
};

pub struct FieldSerializer<'a> {
    rust_field: &'a InputObjectDeriveField,
    graphql_field: &'a InputValue<'a>,
    schema_module: &'a syn::Path,
}

impl<'a> FieldSerializer<'a> {
    pub fn new(
        rust_field: &'a InputObjectDeriveField,
        graphql_field: &'a InputValue<'_>,
        schema_module: &'a syn::Path,
    ) -> FieldSerializer<'a> {
        FieldSerializer {
            rust_field,
            graphql_field,
            schema_module,
        }
    }

    /// Validates the FieldSerializer definition, returning errors if there are
    /// any.
    pub fn validate(&self) -> Option<syn::Error> {
        // First, check for type errors
        if let Err(e) = check_input_types_are_compatible(self.graphql_field, &self.rust_field.ty) {
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

    pub fn type_check(&self) -> TokenStream {
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

        let mut aligned_type = align_input_type(
            &self.rust_field.ty,
            &self.graphql_field.value_type,
            self.graphql_field.has_default,
        );

        TurnLifetimesToAnonymous.visit_type_mut(&mut aligned_type);

        quote! {
            ::cynic::assert_impl!(#aligned_type: #trait_bound);
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
        self.graphql_field.has_default
            && !self.graphql_field.is_nullable()
            && types::outer_type_is_option(&self.rust_field.ty)
    }
}

struct TurnLifetimesToAnonymous;
impl VisitMut for TurnLifetimesToAnonymous {
    fn visit_lifetime_mut(&mut self, i: &mut syn::Lifetime) {
        i.ident = format_ident!("_");
        visit_mut::visit_lifetime_mut(self, i)
    }
}
