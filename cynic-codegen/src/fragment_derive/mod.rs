use std::collections::{HashMap, HashSet};

use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;

use crate::{
    idents::PathExt,
    schema::{
        load_schema,
        types::{self as schema, Type},
        Schema, Unvalidated,
    },
    suggestions::FieldSuggestionError,
    type_validation::{check_spread_type, check_types_are_compatible, CheckMode},
    Errors, FieldType, Ident, TypeIndex,
};

mod arguments;
mod deserialize_impl;
mod fragment_derive_type;
mod fragment_impl;
mod type_ext;

pub(crate) mod input;

use self::{
    arguments::{arguments_from_field_attrs, FieldArgument},
    deserialize_impl::DeserializeImpl,
    fragment_derive_type::FragmentDeriveType,
    fragment_impl::FragmentImpl,
    type_ext::SynTypeExt,
};

pub use input::{FragmentDeriveField, FragmentDeriveInput};

use crate::suggestions::{format_guess, guess_field};

pub fn fragment_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;

    match FragmentDeriveInput::from_derive_input(ast) {
        Ok(input) => {
            let schema_doc = load_schema(&*input.schema_path)
                .map_err(|e| e.into_syn_error(input.schema_path.span()))?;

            let schema = crate::schema::Schema::new(&schema_doc);

            fragment_derive_impl(input, schema).or_else(|e| Ok(e.to_compile_errors()))
        }
        Err(e) => Ok(e.write_errors()),
    }
}

pub fn fragment_derive_impl(
    input: FragmentDeriveInput,
    schema: Schema<'_, Unvalidated>,
) -> Result<TokenStream, Errors> {
    use quote::{quote, quote_spanned};

    let mut input = input;
    input.validate()?;
    input.detect_aliases();

    let schema_path = &input.schema_path;

    let schema_type = schema
        .lookup::<FragmentDeriveType>(&input.graphql_type_name())
        .map_err(|e| syn::Error::new(input.graphql_type_span(), e))?;

    let input_argument_struct = (&input.argument_struct).clone();
    let argument_struct = if let Some(arg_struct) = input_argument_struct {
        let span = arg_struct.span();

        let arg_struct_val: Ident = arg_struct.into();
        let argument_struct = quote_spanned! { span => #arg_struct_val };
        syn::parse2(argument_struct)?
    } else {
        syn::parse2(quote! { () })?
    };

    let graphql_name = &(input.graphql_type_name());
    let schema_module = input.schema_module();
    let ident = input.ident;
    if let darling::ast::Data::Struct(fields) = input.data {
        let fields = pair_fields(fields.iter(), &schema_type)?;

        // TODO: Pair up fields & schema fields here?
        // or do it in FragmentIMpl and expose it?
        let fragment_impl = FragmentImpl::new_for(
            &fields,
            &ident,
            &schema_type,
            &schema_module,
            graphql_name,
            argument_struct,
        )?;

        let deserialize_impl = DeserializeImpl::new(&fields, &ident);

        Ok(quote::quote! {
            #fragment_impl
            #deserialize_impl
        })
    } else {
        Err(syn::Error::new(
            Span::call_site(),
            "QueryFragment can only be derived from a struct".to_string(),
        )
        .into())
    }
}

fn pair_fields<'a, 'b>(
    rust_fields: impl IntoIterator<Item = &'b FragmentDeriveField>,
    schema_type: &'b FragmentDeriveType<'a>,
) -> Result<Vec<(&'b FragmentDeriveField, &'b schema::Field<'a>)>, Errors> {
    let mut result = Vec::new();
    let mut unknown_fields = Vec::new();
    for field in rust_fields {
        let ident = field.graphql_ident();
        match schema_type.field(&ident) {
            Some(schema_field) => result.push((field, schema_field)),
            None => unknown_fields.push(ident),
        }
    }

    if unknown_fields.is_empty() {
        return Ok(result);
    }

    let field_candidates = schema_type
        .fields
        .iter()
        .map(|f| f.name.as_str())
        .collect::<Vec<_>>();

    let errors = unknown_fields
        .into_iter()
        .map(|field| {
            let expected_field = &field.graphql_name();
            let suggested_field = guess_field(field_candidates.iter().copied(), expected_field);
            syn::Error::new(
                field.span(),
                FieldSuggestionError {
                    expected_field,
                    graphql_type_name: schema_type.name,
                    suggested_field,
                },
            )
        })
        .map(Errors::from)
        .collect();

    return Err(errors);
}

#[cfg(test)]
mod tests;
