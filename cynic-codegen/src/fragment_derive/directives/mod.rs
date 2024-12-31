mod output;
mod parsing;

use proc_macro2::Span;
use syn::Ident;

use crate::{
    error::Errors,
    schema::{types::Directive, Schema, Unvalidated},
};

pub use self::{
    output::Output,
    parsing::{directives_from_field_attrs, FieldDirective},
};

use super::arguments::{analyse::AnalysedDirectiveArguments, ArgumentLiteral, FieldArgument};

pub struct AnalysedFieldDirective<'a> {
    directive: Directive<'a>,
    arguments: AnalysedDirectiveArguments<'a>,
}

pub fn process_directive<'a>(
    schema: &'a Schema<'a, Unvalidated>,
    directive: parsing::FieldDirective,
    variables_fields: Option<&syn::Path>,
    span: Span,
) -> Result<AnalysedFieldDirective<'a>, Errors> {
    match directive {
        FieldDirective::Skip(inner) => {
            let Some(directive) = schema.lookup_directive("skip")? else {
                return Err(syn::Error::new(span, "Unknown directive: skip").into());
            };

            let arguments = vec![FieldArgument {
                argument_name: Ident::new("if", span),
                value: super::arguments::FieldArgumentValue::Literal(match inner {
                    parsing::BooleanLiteral::Boolean(value) => {
                        ArgumentLiteral::Literal(syn::Lit::Bool(syn::LitBool { value, span }))
                    }
                    parsing::BooleanLiteral::Variable(var, span) => {
                        ArgumentLiteral::Variable(var, span)
                    }
                }),
            }];

            let arguments = super::arguments::analyse::analyse_directive_arguments(
                schema,
                arguments,
                &directive,
                variables_fields,
                span,
            )?;

            Ok(AnalysedFieldDirective {
                directive,
                arguments,
            })
        }
        FieldDirective::Include(inner) => {
            let Some(directive) = schema.lookup_directive("include")? else {
                return Err(syn::Error::new(span, "Unknown directive: include").into());
            };
            let arguments = vec![FieldArgument {
                argument_name: Ident::new("if", span),
                value: super::arguments::FieldArgumentValue::Literal(match inner {
                    parsing::BooleanLiteral::Boolean(value) => {
                        ArgumentLiteral::Literal(syn::Lit::Bool(syn::LitBool { value, span }))
                    }
                    parsing::BooleanLiteral::Variable(var, span) => {
                        ArgumentLiteral::Variable(var, span)
                    }
                }),
            }];

            let arguments = super::arguments::analyse::analyse_directive_arguments(
                schema,
                arguments,
                &directive,
                variables_fields,
                span,
            )?;

            Ok(AnalysedFieldDirective {
                directive,
                arguments,
            })
        }
        FieldDirective::Other { name, arguments } => {
            let Some(directive) = schema.lookup_directive(&name.to_string())? else {
                return Err(syn::Error::new(span, format!("Unknown directive: {name}")).into());
            };
            let arguments = super::arguments::analyse::analyse_directive_arguments(
                schema,
                arguments,
                &directive,
                variables_fields,
                span,
            )?;

            Ok(AnalysedFieldDirective {
                directive,
                arguments,
            })
        }
    }
}
