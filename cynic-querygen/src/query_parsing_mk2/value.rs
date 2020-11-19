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
pub enum TypedValue<'query, 'schema> {
    Variable(&'query str, InputFieldType<'schema>),
    Int(i64, InputFieldType<'schema>),
    // TODO: consider ordered-float
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
        value: &query::Value<'query>,
        value_type: InputFieldType<'schema>,
        variable_definitions: &[Variable<'query, 'schema>],
    ) -> Result<Self, Error> {
        Ok(match value {
            query::Value::Variable(var_name) => {
                // If this is just a variable then we'll take it's type as our value type.
                // This will proably break on arguments inside lists or objects
                // but I don't have the energy to properly support those right now.
                let value_type = variable_definitions
                    .iter()
                    .find(|var| var.name == *var_name)
                    .ok_or_else(|| {
                        println!("Variables: {:#?}", variable_definitions);
                        Error::UnknownArgument(var_name.to_string())
                    })?
                    .value_type
                    .clone();

                TypedValue::Variable(var_name, value_type)
            }
            query::Value::Int(num) => TypedValue::Int(num.as_i64().unwrap(), value_type),
            query::Value::Float(num) => TypedValue::Float(Decimal::from_f64(*num), value_type),
            query::Value::String(s) => TypedValue::String(s.clone(), value_type),
            query::Value::Boolean(b) => TypedValue::Boolean(*b, value_type),
            query::Value::Null => TypedValue::Null(value_type),
            query::Value::Enum(e) => TypedValue::Enum(e, value_type),
            query::Value::List(values) => {
                let inner_type = value_type.list_inner_type()?;
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
                    value_type,
                )
            }
            query::Value::Object(obj) => {
                if let InputType::InputObject(obj_type) = value_type.inner_ref().lookup()? {
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
                        value_type,
                    )
                } else {
                    return Err(Error::ExpectedInputObject(
                        value_type.inner_name().to_string(),
                    ));
                }
            }
        })
    }

    pub fn value_type(&self) -> &InputFieldType<'schema> {
        match self {
            TypedValue::Variable(_, ty) => ty,
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
        matches!(self, TypedValue::Variable(_, _))
    }

    pub fn variables(&self) -> Vec<Variable<'query, 'schema>> {
        match &self {
            TypedValue::Variable(name, value_type) => vec![Variable {
                name,
                value_type: value_type.clone(),
            }],
            TypedValue::Object(obj_literal, _) => obj_literal
                .iter()
                .flat_map(|(k, v)| v.variables())
                .collect(),
            TypedValue::List(values, _) => values.iter().flat_map(TypedValue::variables).collect(),
            _ => vec![],
        }
    }

    pub fn to_literal(
        &self,
        field_type: &schema::InputFieldType<'schema>,
    ) -> Result<String, Error> {
        use crate::{schema::TypeDefinition, TypeExt};
        use inflector::Inflector;

        // TODO: Simplify this to not require field_type

        Ok(match self {
            TypedValue::Variable(name, _) => {
                if field_type.inner_name() == "String" && field_type.is_required() {
                    // Required String arguments currently take owned Strings,
                    // so we need to clone them.
                    format!("args.{}.clone()", name.to_snake_case())
                } else {
                    // Other arguments we're usually OK taking a reference.
                    format!("&args.{}", name.to_snake_case())
                }
            }
            TypedValue::Int(num, _) => num.to_string(),
            TypedValue::Float(num, _) => num.map(|d| d.to_string()).unwrap_or("null".to_string()),
            TypedValue::String(s, _) => format!("\"{}\".into()", s),
            TypedValue::Boolean(b, _) => b.to_string(),
            TypedValue::Null(_) => "None".into(),
            TypedValue::Enum(v, _) => {
                if let InputType::Enum(en) = field_type.inner_ref().lookup()? {
                    format!("{}::{}", en.name.to_pascal_case(), v.to_pascal_case())
                } else {
                    return Err(Error::ArgumentNotEnum);
                }
            }
            TypedValue::List(values, _) => {
                let inner = values
                    .iter()
                    .map(|v| Ok(v.to_literal(field_type)?))
                    .collect::<Result<Vec<_>, Error>>()?
                    .join(", ");

                format!("vec![{}]", inner)
            }
            TypedValue::Object(object_literal, _) => {
                if let InputType::InputObject(input_object) = field_type.inner_ref().lookup()? {
                    let fields = object_literal
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
                                value.to_literal(&field.value_type)?
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

    pub fn contains_variable(&self) -> bool {
        match self {
            TypedValue::Variable(_, _) => true,
            TypedValue::List(list, _) => list.iter().any(|v| v.contains_variable()),
            TypedValue::Object(obj, _) => obj.iter().any(|(_, v)| v.contains_variable()),
            _ => false,
        }
    }
}
