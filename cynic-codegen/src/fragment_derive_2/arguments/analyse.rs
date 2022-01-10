use std::collections::HashSet;

use proc_macro2::Span;
use syn::Lit;

use super::parsing;
use crate::{
    error::Errors,
    schema::types::{self as schema, InputObjectType, InputType, InputValue, TypeRef},
};

#[derive(Debug, PartialEq)]
pub struct AnalysedArguments<'a> {
    pub schema_field: schema::Field<'a>,
    pub arguments: Vec<Field<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct Object<'a> {
    pub schema_obj: InputObjectType<'a>,
    pub fields: Vec<Field<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct Field<'a> {
    pub schema_field: InputValue<'a>,
    pub value: ArgumentValue<'a>,
}

#[derive(Debug, PartialEq)]
pub enum ArgumentValue<'a> {
    Object(Object<'a>),
    List(Vec<ArgumentValue<'a>>),
    Literal(syn::Lit),
    Bool(bool),
    Variable(Variable<'a>),
    Some(Box<ArgumentValue<'a>>),
    Expression(syn::Expr),
    Null,
}

#[derive(Debug, PartialEq)]
pub struct Variable<'a> {
    pub ident: proc_macro2::Ident,
    pub value_type: TypeRef<'a, InputType<'a>>,
}

pub fn analyse<'a>(
    literals: Vec<parsing::FieldArgument>,
    field: &schema::Field<'a>,
    span: Span,
) -> Result<AnalysedArguments<'a>, Errors> {
    let arguments = analyse_fields(literals, &field.arguments, span)?;

    Ok(AnalysedArguments {
        schema_field: field.clone(),
        arguments,
    })
}

pub fn analyse_fields<'a>(
    literals: Vec<parsing::FieldArgument>,
    arguments: &[InputValue<'a>],
    span: Span,
) -> Result<Vec<Field<'a>>, Errors> {
    validate(&literals, arguments, span)?;

    let mut fields = Vec::new();
    let mut errors = Vec::new();

    for arg in literals {
        let schema_field = arguments
            .iter()
            .find(|a| a.name == arg.argument_name)
            .unwrap();

        match analyse_argument(arg, schema_field) {
            Ok(value) => fields.push(Field {
                schema_field: schema_field.clone(),
                value,
            }),
            Err(e) => errors.push(e),
        }
    }

    if !errors.is_empty() {
        return Err(errors.into_iter().collect());
    }

    return Ok(fields);
}

fn analyse_argument<'a>(
    parsed_arg: parsing::FieldArgument,
    argument: &InputValue<'a>,
) -> Result<ArgumentValue<'a>, Errors> {
    match parsed_arg.value {
        parsing::FieldArgumentValue::Literal(lit) => analyse_value_type(lit, &argument.value_type),
        parsing::FieldArgumentValue::Expression(e) => Ok(ArgumentValue::Expression(e)),
    }
}

fn analyse_value_type<'a>(
    literal: parsing::ArgumentLiteral,
    value_type: &TypeRef<'a, InputType<'a>>,
) -> Result<ArgumentValue<'a>, Errors> {
    use parsing::ArgumentLiteral;

    if let ArgumentLiteral::Variable(ident, _) = literal {
        // We don't ever want to recurse through value_type if we have a variable,
        // as we need the outermost TypeRef to correctly check types.
        return Ok(ArgumentValue::Variable(Variable {
            ident,
            value_type: value_type.clone(),
        }));
    }

    match &value_type {
        TypeRef::Named(_, _, _) => match (value_type.inner_type(), literal) {
            (_, ArgumentLiteral::Variable(_, _)) => {
                // Variable is handled above.
                panic!("This should not happen");
            }

            (InputType::Scalar(_), ArgumentLiteral::Object(_, _)) => {
                todo!("error")
            }
            (InputType::Scalar(_), ArgumentLiteral::List(_, _)) => todo!("error"),
            (InputType::Scalar(_), ArgumentLiteral::Null(_)) => todo!("error"),
            (InputType::Scalar(_), ArgumentLiteral::Literal(lit)) => {
                // TODO: validate this is a valid scalar for the current type
                // Can probably only do that for built in scalars.
                Ok(ArgumentValue::Literal(lit))
            }

            (InputType::Enum(def), ArgumentLiteral::Literal(lit @ Lit::Str(_))) => {
                // TODO: Check that the string is actually a member of the enum
                Ok(ArgumentValue::Literal(lit))
            }
            (InputType::Enum(_), _) => todo!("error"),

            (InputType::InputObject(def), ArgumentLiteral::Object(fields, span)) => {
                let literals = fields.into_iter().collect::<Vec<_>>();
                let fields = analyse_fields(literals, &def.fields, span)?;

                Ok(ArgumentValue::Object(Object {
                    schema_obj: def,
                    fields,
                }))
            }
            (InputType::InputObject(_), _) => todo!("error"),
        },
        TypeRef::List(element_type) => match literal {
            ArgumentLiteral::List(values, _) => {
                let mut output_values = Vec::new();
                let mut errors = Vec::new();
                for value in values {
                    match analyse_value_type(value, element_type.as_ref()) {
                        Ok(v) => output_values.push(v),
                        Err(e) => errors.push(e),
                    }
                }
                if !errors.is_empty() {
                    return Err(errors.into_iter().collect());
                }

                Ok(ArgumentValue::List(output_values))
            }
            other => {
                // Automatically wrap the value in a list per the graphql rules.
                Ok(ArgumentValue::List(vec![analyse_value_type(
                    other,
                    element_type.as_ref(),
                )?]))
            }
        },
        TypeRef::Nullable(inner_typeref) => match literal {
            ArgumentLiteral::Null(_) => Ok(ArgumentValue::Null),
            other => Ok(ArgumentValue::Some(Box::new(analyse_value_type(
                other,
                inner_typeref.as_ref(),
            )?))),
        },
    }
}

fn validate(
    literals: &[parsing::FieldArgument],
    arguments: &[InputValue<'_>],
    span: Span,
) -> Result<(), crate::Errors> {
    let all_args = arguments
        .iter()
        .map(|a| a.name.as_str().to_string())
        .collect::<HashSet<_>>();

    let required_args = arguments
        .iter()
        .filter(|a| !matches!(a.value_type, TypeRef::Nullable(_)))
        .map(|a| a.name.as_str().to_string())
        .collect::<HashSet<_>>();

    let provided_args = literals
        .iter()
        .map(|a| a.argument_name.to_string())
        .collect::<HashSet<_>>();

    let missing_args = required_args
        .difference(&provided_args)
        .cloned()
        .collect::<Vec<_>>();
    let unknown_args = provided_args.difference(&all_args).collect::<Vec<_>>();

    // TODO: Look for duplicates as well?

    let mut errors = Vec::new();
    if !missing_args.is_empty() {
        let fields = missing_args.join(", ");
        errors.push(syn::Error::new(
            span,
            format!("missing fields - {}", fields),
        ));
    }

    for unknown_arg in unknown_args {
        let literal = literals
            .iter()
            .find(|a| a.argument_name == unknown_arg)
            .unwrap();

        // TODO: Ideally I want the span of the name _and_ the value.
        // to match up with what rust does.
        errors.push(syn::Error::new(
            literal.argument_name.span(),
            "no such field",
        ));
    }

    if errors.is_empty() {
        return Ok(());
    }

    Err(errors.into_iter().collect())
}
