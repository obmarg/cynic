mod analyse;
mod output;
mod parsing;

use proc_macro2::Span;

use crate::{
    error::Errors,
    schema::{Schema, Unvalidated},
};

pub use self::{
    output::Output,
    parsing::{arguments_from_field_attrs, FieldArgument},
};

pub fn process_arguments<'a>(
    schema: &Schema<'a, Unvalidated>,
    literals: Vec<parsing::FieldArgument>,
    field: &crate::schema::types::Field<'a>,
    schema_module: syn::Path,
    variables_fields: Option<&syn::Path>,
    span: Span,
) -> Result<Output<'a>, Errors> {
    let analysed = analyse::analyse(schema, literals, field, variables_fields, span)?;

    Ok(Output {
        analysed,
        schema_module,
    })
}

#[cfg(test)]
mod tests;
