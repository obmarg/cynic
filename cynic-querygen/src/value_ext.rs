use graphql_parser::query::Value;
use inflector::Inflector;

use crate::{
    schema::{InputValue, TypeDefinition},
    type_ext::TypeExt,
    Error, TypeIndex,
};

/// Extension trait for graphql_parser::common::Value;
pub trait ValueExt {
    fn to_literal(
        &self,
        input_value: &InputValue<'_>,
        type_definition: &TypeDefinition<'_>,
        type_index: &TypeIndex,
    ) -> Result<String, Error>;
}

impl<'a> ValueExt for Value<'a, &'a str> {
    fn to_literal(
        &self,
        input_value: &InputValue<'_>,
        type_definition: &TypeDefinition<'_>,
        type_index: &TypeIndex,
    ) -> Result<String, Error> {
        Ok(match self {
            Value::Variable(name) => {
                if input_value.value_type.is_required() {
                    // Required arguments don't currently go through IntoArgument, so we need
                    // to manually clone them.
                    format!("args.{}.clone()", name.to_snake_case())
                } else {
                    // Optional types go thorugh IntoArgument so we can take a reference
                    // and trust IntoArgument to convert it to whatever we need.
                    format!("&args.{}", name.to_snake_case())
                }
            }
            Value::Int(num) => num.as_i64().unwrap().to_string(),
            Value::Float(num) => num.to_string(),
            Value::String(s) => format!("\"{}\".into()", s),
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
                    .map(|v| Ok(v.to_literal(input_value, type_definition, type_index)?))
                    .collect::<Result<Vec<_>, Error>>()?
                    .join(", ");

                format!("vec![{}]", inner)
            }
            Value::Object(obj) => {
                if let TypeDefinition::InputObject(input_object) = type_definition {
                    let fields = obj
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

                            let type_name = field.value_type.inner_name();
                            let type_definition = type_index
                                .lookup_type(type_name)
                                .ok_or(Error::UnknownType(type_name.to_string()))?;

                            Ok(format!(
                                "{}: {}",
                                name.to_snake_case(),
                                value.to_literal(field, type_definition, type_index)?
                            ))
                        })
                        .collect::<Result<Vec<_>, Error>>()?
                        .join(", ");

                    format!("{} {{ {} }}", input_object.name.to_pascal_case(), fields)
                } else {
                    return Err(Error::ArgumentNotInputObject);
                }
            }
        })
    }
}
