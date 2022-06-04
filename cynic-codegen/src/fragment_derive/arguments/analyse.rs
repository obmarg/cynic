use std::{collections::HashSet, rc::Rc};

use counter::Counter;
use proc_macro2::Span;
use syn::Lit;

use super::parsing;
use crate::{
    error::Errors,
    schema::types::{self as schema, EnumType, InputObjectType, InputType, InputValue, TypeRef},
};

#[derive(Debug, PartialEq)]
pub struct AnalysedArguments<'a> {
    pub schema_field: schema::Field<'a>,
    pub arguments: Vec<Field<'a>>,
    pub variants: Vec<Rc<VariantDetails<'a>>>,
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
    Variant(Rc<VariantDetails<'a>>),
    Variable(Variable<'a>),
    Some(Box<ArgumentValue<'a>>),
    Expression(syn::Expr),
    Null,
}

#[derive(Debug, PartialEq)]
pub struct Variable<'a> {
    pub ident: proc_macro2::Ident,
    pub value_type: TypeRef<'a, InputType<'a>>,
    pub variable_struct: syn::Path,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct VariantDetails<'a> {
    pub(super) en: EnumType<'a>,
    pub(super) variant: String,
}

pub fn analyse<'a>(
    literals: Vec<parsing::FieldArgument>,
    field: &schema::Field<'a>,
    variables: Option<&syn::Path>,
    span: Span,
) -> Result<AnalysedArguments<'a>, Errors> {
    let mut analysis = Analysis {
        variables,
        variants: HashSet::new(),
    };
    let arguments = analyse_fields(&mut analysis, literals, &field.arguments, span)?;

    let mut variants = analysis.variants.into_iter().collect::<Vec<_>>();
    variants.sort_by_key(|v| (v.en.name, v.variant.clone()));

    Ok(AnalysedArguments {
        schema_field: field.clone(),
        arguments,
        variants,
    })
}

struct Analysis<'schema, 'a> {
    variables: Option<&'a syn::Path>,
    variants: HashSet<Rc<VariantDetails<'schema>>>,
}

impl<'schema, 'a> Analysis<'schema, 'a> {
    fn enum_variant(
        &mut self,
        en: schema::EnumType<'schema>,
        variant: String,
        span: Span,
    ) -> Result<Rc<VariantDetails<'schema>>, Errors> {
        en.values
            .iter()
            .find(|v| v.name == variant)
            .ok_or_else(|| {
                syn::Error::new(
                    span,
                    format!("{} does not have a variant {variant}", en.name),
                )
            })?;

        let variant = Rc::new(VariantDetails { en, variant });

        if let Some(variant) = self.variants.get(&variant) {
            return Ok(Rc::clone(variant));
        }

        self.variants.insert(Rc::clone(&variant));

        Ok(variant)
    }
}

fn analyse_fields<'a>(
    analysis: &mut Analysis<'a, '_>,
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

        match analyse_argument(analysis, arg, schema_field) {
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

    Ok(fields)
}

fn analyse_argument<'a>(
    analysis: &mut Analysis<'a, '_>,
    parsed_arg: parsing::FieldArgument,
    argument: &InputValue<'a>,
) -> Result<ArgumentValue<'a>, Errors> {
    match parsed_arg.value {
        parsing::FieldArgumentValue::Literal(lit) => {
            analyse_value_type(analysis, lit, &argument.value_type)
        }
        parsing::FieldArgumentValue::Expression(e) => Ok(ArgumentValue::Expression(*e)),
    }
}

fn analyse_value_type<'a>(
    analysis: &mut Analysis<'a, '_>,
    literal: parsing::ArgumentLiteral,
    value_type: &TypeRef<'a, InputType<'a>>,
) -> Result<ArgumentValue<'a>, Errors> {
    use parsing::ArgumentLiteral;

    if let ArgumentLiteral::Variable(ident, _) = literal {
        if analysis.variables.is_none() {
            return Err(syn::Error::new(
                ident.span(),
                "You've provided a variable here, but this QueryFragment does not take any variables.  Please add the variables attribute to the struct."
            ).into());
        }

        // We don't ever want to recurse through value_type if we have a variable,
        // as we need the outermost TypeRef to correctly check types.
        return Ok(ArgumentValue::Variable(Variable {
            ident,
            value_type: value_type.clone(),
            variable_struct: analysis.variables.unwrap().clone(),
        }));
    }

    match &value_type {
        TypeRef::Named(_, _, _) => match (value_type.inner_type(), literal) {
            (_, ArgumentLiteral::Variable(_, _)) => {
                // Variable is handled above.
                panic!("This should not happen");
            }

            (InputType::Scalar(_), ArgumentLiteral::Object(_, span)) => {
                Err(syn::Error::new(span, "Expected a scalar here but found an object").into())
            }
            (InputType::Scalar(_), ArgumentLiteral::List(_, span)) => {
                Err(syn::Error::new(span, "Expected a scalar here but found a list").into())
            }
            (InputType::Scalar(_), ArgumentLiteral::Null(span)) => {
                Err(syn::Error::new(span, "Expected a scalar here but found a null").into())
            }
            (InputType::Scalar(_), ArgumentLiteral::Enum(i)) => Err(syn::Error::new(
                i.span(),
                "Expected a scalar here but found an unquoted string",
            )
            .into()),
            (InputType::Scalar(_), ArgumentLiteral::Literal(lit)) => {
                Ok(ArgumentValue::Literal(lit))
            }

            (InputType::Enum(en), ArgumentLiteral::Enum(i)) => Ok(ArgumentValue::Variant(
                analysis.enum_variant(en, i.to_string(), i.span())?,
            )),
            (InputType::Enum(en), ArgumentLiteral::Literal(Lit::Str(s))) => Ok(
                ArgumentValue::Variant(analysis.enum_variant(en, s.value(), s.span())?),
            ),
            (InputType::Enum(_), lit) => {
                Err(syn::Error::new(lit.span(), "Expected an enum here").into())
            }

            (InputType::InputObject(def), ArgumentLiteral::Object(fields, span)) => {
                let literals = fields.into_iter().collect::<Vec<_>>();
                let fields = analyse_fields(analysis, literals, &def.fields, span)?;

                Ok(ArgumentValue::Object(Object {
                    schema_obj: def,
                    fields,
                }))
            }
            (InputType::InputObject(_), lit) => {
                Err(syn::Error::new(lit.span(), "Expected an object here").into())
            }
        },
        TypeRef::List(element_type) => match literal {
            ArgumentLiteral::List(values, _) => {
                let mut output_values = Vec::new();
                let mut errors = Vec::new();
                for value in values {
                    match analyse_value_type(analysis, value, element_type.as_ref()) {
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
                    analysis,
                    other,
                    element_type.as_ref(),
                )?]))
            }
        },
        TypeRef::Nullable(inner_typeref) => match literal {
            ArgumentLiteral::Null(_) => Ok(ArgumentValue::Null),
            other => Ok(ArgumentValue::Some(Box::new(analyse_value_type(
                analysis,
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

    let counts = literals
        .iter()
        .map(|lit| &lit.argument_name)
        .collect::<Counter<_>>();

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

        errors.push(syn::Error::new(
            literal.argument_name.span(),
            "no such field",
        ));
    }

    for (ident, count) in counts {
        if count > 1 {
            errors.push(syn::Error::new(
                ident.span(),
                format!("duplicate field: {ident}"),
            ))
        }
    }

    if errors.is_empty() {
        return Ok(());
    }

    Err(errors.into_iter().collect())
}
