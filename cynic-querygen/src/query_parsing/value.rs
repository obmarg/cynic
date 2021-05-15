use std::collections::BTreeMap;

use cynic_parser::ast::{AstNode, Name, NameOwner, Value};
use rust_decimal::Decimal;

use crate::{
    schema::{InputFieldType, InputType},
    Error,
};

use super::normalisation::Variable;

/// A literal value from a GraphQL query, along with it's type
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypedValue<'schema> {
    Variable {
        name: String,
        value_type: InputFieldType<'schema>,
        field_type: InputFieldType<'schema>,
    },
    Int(i64, InputFieldType<'schema>),
    Float(Option<Decimal>, InputFieldType<'schema>),
    String(String, InputFieldType<'schema>),
    Boolean(bool, InputFieldType<'schema>),
    Null(InputFieldType<'schema>),
    Enum(Name, InputFieldType<'schema>),
    List(Vec<TypedValue<'schema>>, InputFieldType<'schema>),
    Object(BTreeMap<Name, TypedValue<'schema>>, InputFieldType<'schema>),
}

impl<'schema> TypedValue<'schema> {
    pub fn from_query_value(
        value: Value,
        field_type: InputFieldType<'schema>,
        variable_definitions: &[Variable<'schema>],
    ) -> Result<Self, Error> {
        Ok(match value {
            Value::Variable(variable) => {
                let name = variable.name().expect("TODO").to_string();
                // If this is just a variable then we'll take it's type as our value type.
                let value_type = variable_definitions
                    .iter()
                    .find(|var| var.name == name)
                    .ok_or_else(|| Error::UnknownArgument(name.to_string()))?
                    .value_type
                    .clone();

                TypedValue::Variable {
                    name,
                    value_type,
                    field_type,
                }
            }
            Value::IntegerValue(value) => TypedValue::Int(
                value.syntax().to_string().parse().expect("TODO"),
                field_type,
            ),
            Value::FloatValue(value) => {
                TypedValue::Float(value.syntax().to_string().parse().ok(), field_type)
            }
            Value::StringValue(value) => TypedValue::String(
                value.string_contents().expect("TODO").syntax().to_string(),
                field_type,
            ),
            Value::BoolValue(b) => TypedValue::Boolean(b.true_token().is_some(), field_type),
            Value::Null(_) => TypedValue::Null(field_type),
            Value::EnumValue(e) => TypedValue::Enum(e.name().expect("TODO"), field_type),
            Value::ListValue(value) => {
                let inner_type = field_type.list_inner_type()?;
                TypedValue::List(
                    value
                        .values()
                        .map(|val| {
                            TypedValue::from_query_value(
                                val,
                                inner_type.clone(),
                                variable_definitions,
                            )
                        })
                        .collect::<Result<_, Error>>()?,
                    field_type,
                )
            }
            Value::ObjectValue(obj) => {
                if let InputType::InputObject(obj_type) = field_type.inner_ref().lookup()? {
                    TypedValue::Object(
                        obj.fields()
                            .map(|field| {
                                let field_name = field.name().expect("TODO");
                                let schema_field = obj_type
                                    .fields
                                    .iter()
                                    .find(|field| field.name == field_name.text())
                                    .ok_or_else(|| Error::UnknownType(field_name.to_string()))?;

                                Ok((
                                    field_name,
                                    TypedValue::from_query_value(
                                        field.value().expect("TODO"),
                                        schema_field.value_type.clone(),
                                        variable_definitions,
                                    )?,
                                ))
                            })
                            .collect::<Result<_, Error>>()?,
                        field_type,
                    )
                } else {
                    return Err(Error::ExpectedInputObject(
                        field_type.inner_name().to_string(),
                    ));
                }
            }
        })
    }

    pub fn value_type(&self) -> &InputFieldType<'schema> {
        match self {
            TypedValue::Variable { value_type, .. } => value_type,
            TypedValue::Int(_, ty) => ty,
            TypedValue::Float(_, ty) => ty,
            TypedValue::String(_, ty) => ty,
            TypedValue::Boolean(_, ty) => ty,
            TypedValue::Null(ty) => ty,
            TypedValue::Enum(_, ty) => ty,
            TypedValue::List(_, ty) => ty,
            TypedValue::Object(_, ty) => ty,
        }
    }

    pub fn is_variable(&self) -> bool {
        matches!(self, TypedValue::Variable { .. })
    }

    pub fn variables(&self) -> Vec<Variable<'schema>> {
        match &self {
            TypedValue::Variable {
                name, value_type, ..
            } => vec![Variable {
                name: name.clone(),
                value_type: value_type.clone(),
            }],
            TypedValue::Object(obj_literal, _) => obj_literal
                .iter()
                .flat_map(|(_, v)| v.variables())
                .collect(),
            TypedValue::List(values, _) => values.iter().flat_map(TypedValue::variables).collect(),
            _ => vec![],
        }
    }

    pub fn to_literal(&self, context: LiteralContext) -> Result<String, Error> {
        use inflector::Inflector;

        Ok(match self {
            TypedValue::Variable {
                name,
                field_type,
                value_type,
            } => {
                let schema_type = field_type.inner_ref().lookup()?;
                if schema_type.is_definitely_copy() {
                    // Copy types will be implicitly copied so we can just put them literally
                    format!("args.{}", name.to_snake_case())
                } else if context == LiteralContext::Argument {
                    // If we're in argument context then a reference should be OK.
                    // `InputType` usually defines conversions for references.
                    //
                    // There are some cases where this is not true, but can fix
                    // those when they crop up.
                    format!("&args.{}", name.to_snake_case())
                } else {
                    // If this is not in argument position we'll probably need a clone.
                    coerce_variable(
                        field_type,
                        value_type,
                        format!("args.{}.clone()", name.to_snake_case()),
                    )
                }
            }
            TypedValue::Int(num, field_type) => {
                let literal = num.to_string();
                coerce_literal(field_type, context, literal)
            }
            TypedValue::Float(num, field_type) => num
                .map(|d| coerce_literal(field_type, context, d.to_string()))
                .unwrap_or_else(|| "null".to_string()),
            TypedValue::String(s, field_type) => {
                let literal = if field_type.inner_name() == "ID" {
                    format!("cynic::Id::new(\"{}\")", s)
                } else if context == LiteralContext::Argument {
                    // If we're in argument context then InputType gives
                    // us leeway to pass in a string directly.
                    format!("\"{}\"", s)
                } else {
                    // In object or list position we explicitly need to convert to a String
                    format!("\"{}\".to_string()", s)
                };

                coerce_literal(field_type, context, literal)
            }
            TypedValue::Boolean(b, field_type) => {
                coerce_literal(field_type, context, b.to_string())
            }
            TypedValue::Null(_) => "None".into(),
            TypedValue::Enum(v, field_type) => {
                if let InputType::Enum(en) = field_type.inner_ref().lookup()? {
                    let literal = format!(
                        "{}::{}",
                        en.name.to_pascal_case(),
                        v.to_string().to_pascal_case()
                    );

                    coerce_literal(field_type, context, literal)
                } else {
                    return Err(Error::ArgumentNotEnum);
                }
            }
            TypedValue::List(values, _) => {
                let inner = values
                    .iter()
                    .map(|v| v.to_literal(LiteralContext::ListItem))
                    .collect::<Result<Vec<_>, Error>>()?
                    .join(", ");

                format!("vec![{}]", inner)
            }
            TypedValue::Object(object_literal, field_type) => {
                if let InputType::InputObject(input_object) = field_type.inner_ref().lookup()? {
                    let fields = object_literal
                        .iter()
                        .map(|(name, value)| {
                            Ok(format!(
                                "{}: {}",
                                name.to_string().to_snake_case(),
                                value.to_literal(LiteralContext::InputObjectField)?
                            ))
                        })
                        .collect::<Result<Vec<_>, Error>>()?;

                    let fields = fields.join(", ");

                    let literal =
                        format!("{} {{ {} }}", input_object.name.to_pascal_case(), fields);

                    coerce_literal(field_type, context, literal)
                } else {
                    return Err(Error::ArgumentNotInputObject);
                }
            }
        })
    }
}

fn coerce_variable(expected: &InputFieldType, actual: &InputFieldType, input: String) -> String {
    fn inner_fn(
        expected: &InputFieldType,
        actual: &InputFieldType,
        input: String,
        wrap_options: bool,
    ) -> String {
        match (expected, actual) {
            (InputFieldType::NamedType(_), InputFieldType::NamedType(_)) if wrap_options => {
                format!("Some({})", input)
            }
            (InputFieldType::NamedType(_), InputFieldType::NamedType(_)) => input,
            (InputFieldType::ListType(ie), InputFieldType::ListType(ia)) => {
                inner_fn(ie, ia, input, true)
            }
            (InputFieldType::NonNullType(ie), InputFieldType::NonNullType(ia)) => {
                inner_fn(ie, ia, input, false)
            }
            (InputFieldType::ListType(expected), actual) if wrap_options => {
                format!("Some(vec![{}])", inner_fn(expected, actual, input, true))
            }
            (InputFieldType::ListType(expected), actual) => {
                format!("vec![{}]", inner_fn(expected, actual, input, true))
            }
            (InputFieldType::NamedType(_), InputFieldType::NonNullType(_)) => {
                format!("Some({})", input)
            }
            _ => input,
        }
    }

    inner_fn(expected, actual, input, true)
}

fn coerce_literal(into_type: &InputFieldType, context: LiteralContext, input: String) -> String {
    fn inner_fn(into_type: &InputFieldType, wrap_options: bool, input: String) -> String {
        match into_type {
            InputFieldType::NamedType(_) if wrap_options => format!("Some({})", input),
            InputFieldType::NamedType(_) => input,
            InputFieldType::ListType(inner) if wrap_options => {
                format!("Some(vec![{}])", inner_fn(inner, true, input))
            }
            InputFieldType::ListType(inner) => format!("vec![{}]", inner_fn(inner, true, input)),
            InputFieldType::NonNullType(inner) => inner_fn(inner, false, input),
        }
    }

    // If we're in argument context we can rely on IntoArgument to handle
    // Option wrapping so don't do it here.  Non argument contexts need this
    // done here though. Also IntoArgument doesn't currently handle anything
    // with lists though so if a list is present we do the unwrapping here.
    let wrap_outer_options = context != LiteralContext::Argument || into_type.contains_list();

    inner_fn(into_type, wrap_outer_options, input)
}

/// The contexts in which a Value literal can appear in generated code.
///
/// Required because the correct way to express a literal varies depending
/// on it's context.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LiteralContext {
    Argument,
    InputObjectField,
    ListItem,
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::{schema::InputTypeRef, TypeIndex};

    #[test]
    fn variable_option_coercion() {
        let index = Rc::new(TypeIndex::default());
        let variable_type = InputFieldType::NonNullType(Box::new(InputFieldType::NamedType(
            InputTypeRef::test_ref("Int".to_string(), &index),
        )));
        let expected_type =
            InputFieldType::NamedType(InputTypeRef::test_ref("Int".to_string(), &index));

        assert_eq!(
            coerce_variable(
                &expected_type,
                &variable_type,
                "SomeEnum::SomeVariant".into()
            ),
            "Some(SomeEnum::SomeVariant)"
        );

        let expected_type = InputFieldType::NonNullType(Box::new(expected_type));
        assert_eq!(
            coerce_variable(
                &expected_type,
                &variable_type,
                "SomeEnum::SomeVariant".into()
            ),
            "SomeEnum::SomeVariant"
        );
    }

    #[test]
    fn variable_list_coercion() {
        let index = Rc::new(TypeIndex::default());
        let variable_type = InputFieldType::NonNullType(Box::new(InputFieldType::NamedType(
            InputTypeRef::test_ref("Int".to_string(), &index),
        )));
        let expected_type = InputFieldType::ListType(Box::new(InputFieldType::NamedType(
            InputTypeRef::test_ref("Int".to_string(), &index),
        )));

        assert_eq!(
            coerce_variable(
                &expected_type,
                &variable_type,
                "SomeEnum::SomeVariant".into()
            ),
            "Some(vec![Some(SomeEnum::SomeVariant)])"
        );

        let expected_type = InputFieldType::NonNullType(Box::new(expected_type));
        assert_eq!(
            coerce_variable(
                &expected_type,
                &variable_type,
                "SomeEnum::SomeVariant".into()
            ),
            "vec![Some(SomeEnum::SomeVariant)]"
        );

        let expected_type = InputFieldType::NonNullType(Box::new(InputFieldType::ListType(
            Box::new(InputFieldType::NonNullType(Box::new(
                InputFieldType::NamedType(InputTypeRef::test_ref("Int".to_string(), &index)),
            ))),
        )));
        assert_eq!(
            coerce_variable(
                &expected_type,
                &variable_type,
                "SomeEnum::SomeVariant".into()
            ),
            "vec![SomeEnum::SomeVariant]"
        );
    }

    #[test]
    fn literal_coercion_for_fields() {
        let index = Rc::new(TypeIndex::default());
        let expected_type =
            InputFieldType::NamedType(InputTypeRef::test_ref("Int".to_string(), &index));

        assert_eq!(
            coerce_literal(
                &expected_type,
                LiteralContext::InputObjectField,
                "123".to_string()
            ),
            "Some(123)"
        );

        let expected_non_nullable = InputFieldType::NonNullType(Box::new(expected_type.clone()));
        assert_eq!(
            coerce_literal(
                &expected_non_nullable,
                LiteralContext::InputObjectField,
                "123".to_string()
            ),
            "123"
        );

        let expected_type = InputFieldType::ListType(Box::new(expected_type));

        assert_eq!(
            coerce_literal(
                &expected_type,
                LiteralContext::InputObjectField,
                "123".to_string()
            ),
            "Some(vec![Some(123)])"
        );

        let expected_type = InputFieldType::NonNullType(Box::new(expected_type));
        assert_eq!(
            coerce_literal(
                &expected_type,
                LiteralContext::InputObjectField,
                "123".to_string()
            ),
            "vec![Some(123)]"
        );
    }

    #[test]
    fn literal_coercion_for_arguments() {
        let index = Rc::new(TypeIndex::default());
        let expected_type =
            InputFieldType::NamedType(InputTypeRef::test_ref("Int".to_string(), &index));

        assert_eq!(
            coerce_literal(&expected_type, LiteralContext::Argument, "123".to_string()),
            "123"
        );

        let expected_non_nullable = InputFieldType::NonNullType(Box::new(expected_type.clone()));
        assert_eq!(
            coerce_literal(
                &expected_non_nullable,
                LiteralContext::Argument,
                "123".to_string()
            ),
            "123"
        );

        let expected_type = InputFieldType::ListType(Box::new(expected_type));

        assert_eq!(
            coerce_literal(&expected_type, LiteralContext::Argument, "123".to_string()),
            "Some(vec![Some(123)])"
        );

        let expected_type = InputFieldType::NonNullType(Box::new(expected_type));
        assert_eq!(
            coerce_literal(&expected_type, LiteralContext::Argument, "123".to_string()),
            "vec![Some(123)]"
        );
    }
}
