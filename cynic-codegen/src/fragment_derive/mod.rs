use proc_macro2::{Span, TokenStream};

use crate::{
    schema::{
        load_schema,
        types::{self as schema},
        Schema, Unvalidated,
    },
    suggestions::FieldSuggestionError,
    Errors,
};

mod arguments;
mod deserialize_impl;
mod fragment_derive_type;
mod fragment_impl;
mod type_ext;

pub(crate) mod input;

use self::{
    deserialize_impl::DeserializeImpl, fragment_derive_type::FragmentDeriveType,
    fragment_impl::FragmentImpl,
};

pub use input::{FragmentDeriveField, FragmentDeriveInput};

use crate::suggestions::guess_field;

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
    let mut input = input;
    input.validate()?;
    input.detect_aliases();

    let schema_type = schema
        .lookup::<FragmentDeriveType<'_>>(&input.graphql_type_name())
        .map_err(|e| syn::Error::new(input.graphql_type_span(), e))?;

    let graphql_name = &(input.graphql_type_name());
    let schema_module = input.schema_module();
    let variables = input.variables();
    let deprecations = input.deprecations();
    if let darling::ast::Data::Struct(fields) = input.data {
        let fields = pair_fields(fields.iter(), &schema_type)?;

        let fragment_impl = FragmentImpl::new_for(
            &schema,
            &fields,
            &input.ident,
            &input.generics,
            &schema_type,
            &schema_module,
            graphql_name,
            variables.as_ref(),
        )?;

        let deserialize_impl = DeserializeImpl::new(&fields, &input.ident, &input.generics);

        Ok(quote::quote! {
            #fragment_impl
            #deserialize_impl
            #deprecations
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
) -> Result<Vec<(&'b FragmentDeriveField, Option<&'b schema::Field<'a>>)>, Errors> {
    let mut result = Vec::new();
    let mut unknown_fields = Vec::new();
    for field in rust_fields {
        let ident = field.graphql_ident();
        match (schema_type.field(&ident), *field.spread) {
            (Some(schema_field), _) => result.push((field, Some(schema_field))),
            (None, false) => unknown_fields.push(ident),
            (None, true) => result.push((field, None)),
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
                    graphql_type_name: schema_type.name.as_ref(),
                    suggested_field,
                },
            )
        })
        .map(Errors::from)
        .collect();

    Err(errors)
}

#[cfg(test)]
mod tests;
