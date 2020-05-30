use graphql_parser::query::Value;

/// Extension trait for graphql_parser::common::Value;
pub trait ValueExt {
    fn to_literal(&self) -> String;
}

impl<'a> ValueExt for Value<'a, &'a str> {
    fn to_literal(&self) -> String {
        match self {
            Value::Variable(name) => format!("args.{}", name),
            Value::Int(num) => num.as_i64().unwrap().to_string(),
            Value::Float(num) => num.to_string(),
            Value::String(s) => format!("\"{}\".to_string()", s),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "None".into(),
            Value::Enum(v) => v.to_string(),
            Value::List(values) => {
                let inner = values
                    .iter()
                    .map(|v| v.to_literal())
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("vec![{}]", inner)
            }
            Value::Object(_) => "TODO".into(),
        }
    }
}
