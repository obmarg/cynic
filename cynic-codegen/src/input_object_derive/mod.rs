use proc_macro2::{Span, TokenStream};

use crate::{
    error::Errors,
    idents::RenameAll,
    input_object_derive::{
        one_of::oneof_input_object_derive, standard::standard_input_object_derive,
    },
    schema::{Schema, types::InputObjectType},
};

mod one_of;
mod standard;

pub(crate) mod input;

mod pairing;
#[cfg(test)]
mod tests;

pub use input::InputObjectDeriveInput;

pub fn input_object_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;

    let struct_span = ast.ident.span();

    match InputObjectDeriveInput::from_derive_input(ast) {
        Ok(input) => {
            input_object_derive_impl(input, struct_span).or_else(|e| Ok(e.to_compile_errors()))
        }
        Err(e) => Ok(e.write_errors()),
    }
}

pub fn input_object_derive_impl(
    input: InputObjectDeriveInput,
    struct_span: Span,
) -> Result<TokenStream, Errors> {
    let schema = Schema::new(input.schema_input()?);

    let input_object = schema
        .lookup::<InputObjectType<'_>>(&input.graphql_type_name())
        .map_err(|e| syn::Error::new(input.graphql_type_span(), e))?;

    let rename_all = input.rename_all.unwrap_or(RenameAll::CamelCase);

    match &input.data {
        darling::ast::Data::Enum(variants) => {
            if input_object.fields.iter().any(|field| !field.is_nullable()) {
                return Err(syn::Error::new(
                    struct_span,
                    "An enum can only represent an InputObject when all of the input fields are nullable".to_string(),
                )
                .into());
            }
            oneof_input_object_derive(
                &input,
                struct_span,
                &schema,
                input_object,
                rename_all,
                variants,
            )
        }
        darling::ast::Data::Struct(fields) => standard_input_object_derive(
            &input,
            struct_span,
            &schema,
            input_object,
            rename_all,
            fields,
        ),
    }
}
