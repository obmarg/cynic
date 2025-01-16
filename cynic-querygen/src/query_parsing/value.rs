use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::collections::BTreeMap;

use crate::{
    schema::{InputFieldType, InputType},
    Error,
};

use super::normalisation::Variable;

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
        value: cynic_parser::Value<'query>,
        field_type: InputFieldType<'schema>,
        variable_definitions: &[Variable<'query, 'schema>],
    ) -> Result<Self, Error> {
        Ok(match value {
            cynic_parser::Value::Variable(variable) => {
                let name = variable.name();

                dbg!(variable_definitions);

                // If this is just a variable then we'll take it's type as our value type.
                let value_type = variable_definitions
                    .iter()
                    .find(|var| var.name == name)
                    .ok_or_else(|| dbg!(Error::UnknownArgument(name.to_string())))?
                    .value_type
                    .clone();

                TypedValue::Variable {
                    name,
                    value_type,
                    field_type,
                }
            }
            cynic_parser::Value::Int(num) => TypedValue::Int(num.as_i64(), field_type),
            cynic_parser::Value::Float(num) => {
                TypedValue::Float(Decimal::from_f64(num.as_f64()), field_type)
            }
            cynic_parser::Value::String(s) => TypedValue::String(s.to_string(), field_type),
            cynic_parser::Value::Boolean(b) => TypedValue::Boolean(b.value(), field_type),
            cynic_parser::Value::Null(_) => TypedValue::Null(field_type),
            cynic_parser::Value::Enum(e) => TypedValue::Enum(e.name(), field_type),
            cynic_parser::Value::List(values) => {
                let inner_type = field_type.list_inner_type()?;
                TypedValue::List(
                    values
                        .items()
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
            cynic_parser::Value::Object(obj) => {
                if let InputType::InputObject(obj_type) = field_type.inner_ref().lookup()? {
                    TypedValue::Object(
                        obj.fields()
                            .map(|query_field| {
                                let field_name = query_field.name();

                                let schema_field = obj_type
                                    .fields
                                    .iter()
                                    .find(|schema_field| schema_field.name == field_name)
                                    .ok_or_else(|| Error::UnknownType(field_name.to_string()))?;

                                Ok((
                                    field_name,
                                    TypedValue::from_query_value(
                                        query_field.value(),
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
