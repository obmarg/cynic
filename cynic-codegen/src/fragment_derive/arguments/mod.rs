mod analyse;
mod output;
mod parsing;

use proc_macro2::Span;

use crate::error::Errors;

pub use self::{
    output::Output,
    parsing::{arguments_from_field_attrs, FieldArgument},
};

pub fn process_arguments<'a>(
    literals: Vec<parsing::FieldArgument>,
    field: &crate::schema::types::Field<'a>,
    schema_module: syn::Path,
    argument_struct: Option<&syn::Ident>,
    span: Span,
) -> Result<Output<'a>, Errors> {
    let analysed = analyse::analyse(literals, field, argument_struct, span)?;

    Ok(Output {
        analysed,
        schema_module,
        argument_struct: argument_struct.cloned(),
    })
}

#[cfg(test)]
mod tests;
