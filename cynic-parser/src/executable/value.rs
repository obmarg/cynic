use crate::AstLookup;

use super::{
    ids::{StringId, ValueId},
    ExecutableId, ReadContext,
};

pub enum ValueRecord {
    Variable(StringId),
    Int(i32),
    Float(f32),
    String(StringId),
    Boolean(bool),
    Null,
    Enum(StringId),

    // TODO: Figure out how to express these as IdRange
    // or similar.
    List(Vec<ValueId>),
    Object(Vec<(StringId, ValueId)>),
}

#[derive(Clone)]
pub enum Value<'a> {
    Variable(&'a str),
    Int(i32),
    Float(f32),
    String(&'a str),
    Boolean(bool),
    Null,
    Enum(&'a str),
    List(Vec<Value<'a>>),
    Object(Vec<(&'a str, Value<'a>)>),
}

impl PartialEq for Value<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Variable(a), Value::Variable(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Enum(a), Value::Enum(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => {
                a.len() == b.len()
                    && a.iter().all(|(name, value)| {
                        let Some((_, b_value)) = b.iter().find(|(b_name, _)| b_name == name) else {
                            return false;
                        };

                        value == b_value
                    })
            }
            _ => false,
        }
    }
}

impl ExecutableId for ValueId {
    type Reader<'a> = Value<'a>;
}

impl<'a> From<ReadContext<'a, ValueId>> for Value<'a> {
    fn from(reader: ReadContext<'a, ValueId>) -> Self {
        let ast = &reader.document;

        match ast.lookup(reader.id) {
            ValueRecord::Variable(id) => Value::Variable(ast.lookup(*id)),
            ValueRecord::Int(num) => Value::Int(*num),
            ValueRecord::Float(num) => Value::Float(*num),
            ValueRecord::String(id) => Value::String(ast.lookup(*id)),
            ValueRecord::Boolean(val) => Value::Boolean(*val),
            ValueRecord::Null => Value::Null,
            ValueRecord::Enum(id) => Value::Enum(ast.lookup(*id)),
            ValueRecord::List(ids) => Value::List(ids.iter().map(|id| ast.read(*id)).collect()),
            ValueRecord::Object(pairs) => Value::Object(
                pairs
                    .iter()
                    .map(|(name, value)| (ast.lookup(*name), ast.read(*value)))
                    .collect(),
            ),
        }
    }
}
