use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::collections::BTreeMap;

use crate::{
    query,
    schema::{self, InputType},
    Error, TypeIndex,
};

/// A GraphQL value.
///
/// We redefine this so we can map `Float(f64)` to Float(Option<Decimal>)
/// which allows us to derive Ord, Hash & Eq
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value<'query> {
    Variable(&'query str),
    Int(i64),
    // TODO: consider ordered-float
    Float(Option<Decimal>),
    String(String),
    Boolean(bool),
    Null,
    Enum(&'query str),
    List(Vec<Value<'query>>),
    Object(BTreeMap<&'query str, Value<'query>>),
}

impl<'query> Value<'query> {
    pub fn to_literal<'schema>(
        &self,
        field_type: &schema::InputFieldType<'schema>,
    ) -> Result<String, Error> {
        use crate::{schema::TypeDefinition, TypeExt};
        use inflector::Inflector;

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
                    .map(|v| Ok(v.to_literal(field_type)?))
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
                                value.to_literal(&field.value_type)?
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
}

impl<'query> From<&query::Value<'query>> for Value<'query> {
    fn from(val: &query::Value<'query>) -> Value<'query> {
        match val {
            query::Value::Variable(var) => Value::Variable(var),
            query::Value::Int(num) => Value::Int(num.as_i64().unwrap()),
            query::Value::Float(num) => Value::Float(Decimal::from_f64(*num)),
            query::Value::String(s) => Value::String(s.clone()),
            query::Value::Boolean(b) => Value::Boolean(*b),
            query::Value::Null => Value::Null,
            query::Value::Enum(e) => Value::Enum(e),
            query::Value::List(values) => Value::List(values.iter().map(Into::into).collect()),
            query::Value::Object(obj) => {
                Value::Object(obj.iter().map(|(k, v)| (*k, Value::from(v))).collect())
            }
        }
    }
}
