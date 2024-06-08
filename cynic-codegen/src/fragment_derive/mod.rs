use proc_macro2::TokenStream;

use crate::{
    schema::{
        types::{self as schema, OutputType},
        Schema,
    },
    suggestions::FieldSuggestionError,
    Errors,
};

mod arguments;
mod deserialize_impl;
mod directives;
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
        Ok(input) => fragment_derive_impl(input).or_else(|e| Ok(e.to_compile_errors())),
        Err(e) => Ok(e.write_errors()),
    }
}

pub fn fragment_derive_impl(input: FragmentDeriveInput) -> Result<TokenStream, Errors> {
    let mut input = input;

    input.detect_aliases();
    let fields = input.validate()?;

    let schema = Schema::new(input.schema_input()?);

    let schema_type = schema
        .lookup::<FragmentDeriveType<'_>>(&input.graphql_type_name())
        .map_err(|e| syn::Error::new(input.graphql_type_span(), e))?;

    let graphql_name = &(input.graphql_type_name());
    let schema_module = input.schema_module();
    let variables = input.variables();
    let fields = pair_fields(fields.into_iter(), &schema_type)?;

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

    let field_module_path = schema_type.field_module.to_path(&schema_module);

    let deserialize_impl = DeserializeImpl::new(
        &schema,
        &fields,
        &input.ident,
        &input.generics,
        &field_module_path,
    );

    Ok(quote::quote! {
        #fragment_impl
        #deserialize_impl
    })
}

fn pair_fields<'a>(
    rust_fields: impl IntoIterator<Item = FragmentDeriveField>,
    schema_type: &FragmentDeriveType<'a>,
) -> Result<Vec<(FragmentDeriveField, Option<schema::Field<'a>>)>, Errors> {
    let mut result = Vec::new();
    let mut unknown_fields = Vec::new();
    for field in rust_fields {
        let ident = field.graphql_ident();
        match (schema_type.field(&ident), field.spread()) {
            (Some(schema_field), _) => result.push((field, Some(schema_field.clone()))),
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

impl OutputType<'_> {
    fn as_kind(&self) -> FieldKind {
        match self {
            OutputType::Scalar(_) => FieldKind::Scalar,
            OutputType::Enum(_) => FieldKind::Enum,
            OutputType::Object(_) => FieldKind::Composite,
            OutputType::Interface(_) => FieldKind::Interface,
            OutputType::Union(_) => FieldKind::Union,
        }
    }
}

#[derive(Clone, Copy)]
enum FieldKind {
    Composite,
    Scalar,
    Enum,
    Interface,
    Union,
}

#[cfg(test)]
mod tests;
