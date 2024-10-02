use std::fmt;

use crate::{printing::escape_string, values::Value};

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(val) => write!(f, "{}", val),
            Value::Float(val) => write!(f, "{}", val),
            Value::Boolean(val) => write!(f, "{}", val),
            Value::String(val) => {
                let val = escape_string(val.value());
                write!(f, "\"{val}\"")
            }
            Value::Object(object) => {
                write!(f, "{{")?;
                for (i, field) in object.fields().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field.name(), field.value())?;
                }
                write!(f, "}}")
            }
            Value::List(vals) => {
                write!(f, "[")?;
                for (i, val) in vals.items().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            Value::Variable(variable) => {
                write!(f, "${}", variable.name())
            }
            Value::Null(_) => {
                write!(f, "null")
            }
            Value::Enum(name) => {
                write!(f, "{name}")
            }
        }
    }
}
