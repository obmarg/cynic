use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::collections::BTreeMap;

use crate::query;

/// A GraphQL value.
///
/// We redefine this so we can map `Float(f64)` to Float(Option<Decimal>)
/// which allows us to derive Ord, Hash & Eq
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value<'query> {
    Variable(&'query str),
    Int(i64),
    Float(Option<Decimal>),
    String(String),
    Boolean(bool),
    Null,
    Enum(&'query str),
    List(Vec<Value<'query>>),
    Object(BTreeMap<&'query str, Value<'query>>),
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
