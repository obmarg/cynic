use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::collections::BTreeMap;

use crate::{
    schema::{InputFieldType, InputType},
    Error,
};

use super::{normalisation::Variable, parser};

/// A literal value from a GraphQL query, along with it's type
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypedValue<'query, 'schema> {
    Variable {
        name: &'query str,
        value_type: InputFieldType<'schema>,
        field_type: InputFieldType<'schema>,
    },
    Int(i64, InputFieldType<'schema>),
    Float(Option<Decimal>, InputFieldType<'schema>),
    String(String, InputFieldType<'schema>),
    Boolean(bool, InputFieldType<'schema>),
    Null(InputFieldType<'schema>),
    Enum(&'query str, InputFieldType<'schema>),
    List(Vec<TypedValue<'query, 'schema>>, InputFieldType<'schema>),
    Object(
        BTreeMap<&'query str, TypedValue<'query, 'schema>>,
        InputFieldType<'schema>,
    ),
}

impl<'query, 'schema> TypedValue<'query, 'schema> {
    pub fn from_query_value(
        value: &parser::Value<'query>,
        field_type: InputFieldType<'schema>,
        variable_definitions: &[Variable<'query, 'schema>],
    ) -> Result<Self, Error> {
        Ok(match value {
            parser::Value::Variable(name) => {
                // If this is just a variable then we'll take it's type as our value type.
                let value_type = variable_definitions
                    .iter()
                    .find(|var| var.name == *name)
                    .ok_or_else(|| Error::UnknownArgument(name.to_string()))?
                    .value_type
                    .clone();

                TypedValue::Variable {
                    name,
                    value_type,
                    field_type,
                }
            }
            parser::Value::Int(num) => TypedValue::Int(num.as_i64().unwrap(), field_type),
            parser::Value::Float(num) => TypedValue::Float(Decimal::from_f64(*num), field_type),
            parser::Value::String(s) => TypedValue::String(s.clone(), field_type),
            parser::Value::Boolean(b) => TypedValue::Boolean(*b, field_type),
            parser::Value::Null => TypedValue::Null(field_type),
            parser::Value::Enum(e) => TypedValue::Enum(e, field_type),
            parser::Value::List(values) => {
                let inner_type = field_type.list_inner_type()?;
                TypedValue::List(
                    values
                        .iter()
                        .map(|val| {
                            Ok(TypedValue::from_query_value(
                                val,
                                inner_type.clone(),
                                variable_definitions,
                            )?)
                        })
                        .collect::<Result<_, Error>>()?,
                    field_type,
                )
            }
            parser::Value::Object(obj) => {
                if let InputType::InputObject(obj_type) = field_type.inner_ref().lookup()? {
                    TypedValue::Object(
                        obj.iter()
                            .map(|(k, v)| {
                                let field = obj_type
                                    .fields
                                    .iter()
                                    .find(|field| field.name == *k)
                                    .ok_or_else(|| Error::UnknownType(k.to_string()))?;

                                Ok((
                                    *k,
                                    TypedValue::from_query_value(
                                        v,
                                        field.value_type.clone(),
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
        matches!(self, TypedValue::Variable{ ..})
    }

    pub fn variables(&self) -> Vec<Variable<'query, 'schema>> {
        match &self {
            TypedValue::Variable {
                name, value_type, ..
            } => vec![Variable {
                name,
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
                } else if context == LiteralContext::Argument
                    && (!field_type.is_required() || schema_type.is_input_object())
                {
                    // Optional arguments or input types are OK to take a reference.
                    format!("&args.{}", name.to_snake_case())
                } else {
                    // If this is a required arg _or_ not in argument position we'll need a clone
                    let clone_field = format!("args.{}.clone()", name.to_snake_case());
                    if value_type.is_required() && !field_type.is_required() {
                        // Our value is not an Option but the field _is_ so wrap it in Some
                        format!("Some({})", clone_field)
                    } else {
                        clone_field
                    }
                }
            }
            TypedValue::Int(num, _) => num.to_string(),
            TypedValue::Float(num, _) => num
                .map(|d| d.to_string())
                .unwrap_or_else(|| "null".to_string()),
            TypedValue::String(s, field_type) => {
                if field_type.inner_name() == "ID" {
                    format!("cynic::Id::new(\"{}\")", s)
                } else {
                    format!("\"{}\".into()", s)
                }
            }
            TypedValue::Boolean(b, _) => b.to_string(),
            TypedValue::Null(_) => "None".into(),
            TypedValue::Enum(v, field_type) => {
                if let InputType::Enum(en) = field_type.inner_ref().lookup()? {
                    format!("{}::{}", en.name.to_pascal_case(), v.to_pascal_case())
                } else {
                    return Err(Error::ArgumentNotEnum);
                }
            }
            TypedValue::List(values, _) => {
                let inner = values
                    .iter()
                    .map(|v| Ok(v.to_literal(LiteralContext::ListItem)?))
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
                                name.to_snake_case(),
                                value.to_literal(LiteralContext::InputObjectField)?
                            ))
                        })
                        .collect::<Result<Vec<_>, Error>>()?;

                    let fields = fields.join(", ");

                    format!("{} {{ {} }}", input_object.name.to_pascal_case(), fields)
                } else {
                    return Err(Error::ArgumentNotInputObject);
                }
            }
        })
    }
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
