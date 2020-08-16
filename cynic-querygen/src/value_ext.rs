use graphql_parser::query::Value;
use inflector::Inflector;

use crate::schema::TypeDefinition;
use crate::Error;

/// Extension trait for graphql_parser::common::Value;
pub trait ValueExt {
    fn to_literal(&self, type_definition: &TypeDefinition<'_>) -> Result<String, Error>;
}

impl<'a> ValueExt for Value<'a, &'a str> {
    fn to_literal(&self, type_definition: &TypeDefinition<'_>) -> Result<String, Error> {
        Ok(match self {
            Value::Variable(name) => format!("args.{}", name.to_snake_case()),
            Value::Int(num) => num.as_i64().unwrap().to_string(),
            Value::Float(num) => num.to_string(),
            Value::String(s) => format!("\"{}\".to_string()", s),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "None".into(),
            Value::Enum(v) => {
                if let TypeDefinition::Enum(en) = type_definition {
                    format!("{}::{}", en.name.to_pascal_case(), v.to_pascal_case())
                } else {
                    return Err(Error::ArgumentNotEnum);
                }
            }
            Value::List(values) => {
                let inner = values
                    .iter()
                    .map(|v| Ok(v.to_literal(type_definition)?))
                    .collect::<Result<Vec<_>, Error>>()?
                    .join(", ");

                format!("vec![{}]", inner)
            }
            Value::Object(_) => "TODO".into(),
        })
    }
}
