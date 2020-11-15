use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::collections::BTreeMap;

use crate::{
    query,
    schema::{self, InputFieldType, InputType},
    Error, TypeIndex,
};

use super::normalisation::Variable;

/// A literal value from a GraphQL query, along with it's type
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypedValue<'query, 'schema> {
    pub value: Value<'query, 'schema>,
    pub value_type: InputFieldType<'schema>,
}

impl<'query, 'schema> TypedValue<'query, 'schema> {
    pub fn from_query_value(
        value: &query::Value<'query>,
        value_type: InputFieldType<'schema>,
        variable_definitions: &[Variable<'query, 'schema>],
    ) -> Result<Self, Error> {
        let value_type = match value {
            query::Value::Variable(var_name) => {
                // If this is just a variable then we'll take it's type as our value type.
                // This will proably break on arguments inside lists or objects
                // but I don't have the energy to properly support those right now.
                variable_definitions
                    .iter()
                    .find(|var| var.name == *var_name)
                    .ok_or_else(|| Error::UnknownArgument(var_name.to_string()))?
                    .value_type
                    .clone()
            }
            _ => value_type,
        };

        Ok(TypedValue {
            value: Value::from_query_value(value, value_type.clone())?,
            value_type,
        })
    }

    pub fn is_variable(&self) -> bool {
        matches!(self.value, Value::Variable(_))
    }

    pub fn contains_variable(&self) -> bool {
        self.value.contains_variable()
    }

    pub fn variables(&self) -> Vec<Variable<'query, 'schema>> {
        match &self.value {
            Value::Variable(name) => vec![Variable {
                name,
                value_type: self.value_type.clone(),
            }],
            Value::Object(obj_literal) => obj_literal
                .iter()
                .flat_map(|(k, v)| v.variables())
                .collect(),
            Value::List(values) => values.iter().flat_map(TypedValue::variables).collect(),
            _ => vec![],
        }
    }
}

/// A GraphQL value.
///
/// We redefine this so we can map `Float(f64)` to Float(Option<Decimal>)
/// which allows us to derive Ord, Hash & Eq
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value<'query, 'schema> {
    Variable(&'query str),
    Int(i64),
    // TODO: consider ordered-float
    Float(Option<Decimal>),
    String(String),
    Boolean(bool),
    Null,
    Enum(&'query str),
    List(Vec<TypedValue<'query, 'schema>>),
    Object(BTreeMap<&'query str, TypedValue<'query, 'schema>>),
}

impl<'query, 'schema> Value<'query, 'schema> {
    pub fn to_literal(
        &self,
        field_type: &schema::InputFieldType<'schema>,
    ) -> Result<String, Error> {
        use crate::{schema::TypeDefinition, TypeExt};
        use inflector::Inflector;

        // TODO: Simplify this to not require field_type

        Ok(match self {
            Value::Variable(name) => {
                if field_type.inner_name() == "String" && field_type.is_required() {
                    // Required String arguments currently take owned Strings,
                    // so we need to clone them.
                    format!("args.{}.clone()", name.to_snake_case())
                } else {
                    // Other arguments we're usually OK taking a reference.
                    format!("&args.{}", name.to_snake_case())
                }
            }
            Value::Int(num) => num.to_string(),
            Value::Float(num) => num.map(|d| d.to_string()).unwrap_or("null".to_string()),
            Value::String(s) => format!("\"{}\".into()", s),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "None".into(),
            Value::Enum(v) => {
                if let InputType::Enum(en) = field_type.inner_ref().lookup()? {
                    format!("{}::{}", en.name.to_pascal_case(), v.to_pascal_case())
                } else {
                    return Err(Error::ArgumentNotEnum);
                }
            }
            Value::List(values) => {
                let inner = values
                    .iter()
                    .map(|v| Ok(v.value.to_literal(field_type)?))
                    .collect::<Result<Vec<_>, Error>>()?
                    .join(", ");

                format!("vec![{}]", inner)
            }
            Value::Object(object_literal) => {
                if let InputType::InputObject(input_object) = field_type.inner_ref().lookup()? {
                    let mut fields = object_literal
                        .iter()
                        .map(|(name, value)| {
                            let field = input_object
                                .fields
                                .iter()
                                .find(|field| field.name == *name)
                                .ok_or(Error::UnknownField(
                                    name.to_string(),
                                    input_object.name.to_string(),
                                ))?;

                            Ok(format!(
                                "{}: {}",
                                name.to_snake_case(),
                                value.value.to_literal(&field.value_type)?
                            ))
                        })
                        .collect::<Result<Vec<_>, Error>>()?;

                    for object_field in &input_object.fields {
                        if !object_literal.contains_key(object_field.name)
                            && !object_field.value_type.is_required()
                        {
                            // If this field is not in the literal and is not required we default it
                            // to None.  If it _is_ required, we let the rust compiler warn about that
                            // in the generated code.
                            fields.push(format!("{}: None", object_field.name.to_snake_case()));
                        }
                    }

                    let fields = fields.join(", ");

                    format!("{} {{ {} }}", input_object.name.to_pascal_case(), fields)
                } else {
                    return Err(Error::ArgumentNotInputObject);
                }
            }
        })
    }

    fn contains_variable(&self) -> bool {
        match self {
            Value::Variable(_) => true,
            Value::List(list) => list.iter().any(|v| v.contains_variable()),
            Value::Object(obj) => obj.iter().any(|(_, v)| v.contains_variable()),
            _ => false,
        }
    }
}

impl<'query, 'schema> Value<'query, 'schema> {
    fn from_query_value(
        val: &query::Value<'query>,
        value_type: InputFieldType<'schema>,
    ) -> Result<Value<'query, 'schema>, Error> {
        Ok(match val {
            query::Value::Variable(var) => Value::Variable(var),
            query::Value::Int(num) => Value::Int(num.as_i64().unwrap()),
            query::Value::Float(num) => Value::Float(Decimal::from_f64(*num)),
            query::Value::String(s) => Value::String(s.clone()),
            query::Value::Boolean(b) => Value::Boolean(*b),
            query::Value::Null => Value::Null,
            query::Value::Enum(e) => Value::Enum(e),
            query::Value::List(values) => {
                let inner_type = value_type.list_inner_type()?;
                Value::List(
                    values
                        .iter()
                        .map(|val| Ok(TypedValue::from_query_value(val, inner_type.clone(), &[])?))
                        .collect::<Result<_, Error>>()?,
                )
            }
            query::Value::Object(obj) => {
                if let InputType::InputObject(obj_type) = value_type.inner_ref().lookup()? {
                    Value::Object(
                        obj.iter()
                            .map(|(k, v)| {
                                let field = obj_type
                                    .fields
                                    .iter()
                                    .find(|field| field.name == *k)
                                    .ok_or_else(|| Error::UnknownType(k.to_string()))?;

                                Ok((
                                    *k,
                                    TypedValue::from_query_value(v, field.value_type.clone(), &[])?,
                                ))
                            })
                            .collect::<Result<_, Error>>()?,
                    )
                } else {
                    return Err(Error::ExpectedInputObject(
                        value_type.inner_name().to_string(),
                    ));
                }
            }
        })
    }
}
