use crate::{common::IntValue, type_system::ids::ValueId, AstLookup};

use super::{BlockStringLiteralId, StringId, TypeSystemId};

pub enum ValueRecord {
    Variable(StringId),
    Int(i64),
    Float(f32),
    String(StringId),
    BlockString(BlockStringLiteralId),
    Boolean(bool),
    Null,
    Enum(StringId),
    List(Vec<ValueId>),
    Object(Vec<(StringId, ValueId)>),
}

#[derive(Clone, Debug)]
pub enum Value<'a> {
    Variable(&'a str),
    Int(IntValue),
    Float(f32),
    String(&'a str),
    BlockString(&'a str),
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
            (Value::String(a), Value::BlockString(b))
            | (Value::String(a), Value::String(b))
            | (Value::BlockString(a), Value::String(b))
            | (Value::BlockString(a), Value::BlockString(b)) => a == b,
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

impl<'a> Value<'a> {
    pub fn as_str(&self) -> Option<&'a str> {
        match self {
            Self::String(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_list_iter(&self) -> Option<impl ExactSizeIterator<Item = Value<'a>>> {
        match self {
            Self::List(inner) => Some(inner.clone().into_iter()),
            _ => None,
        }
    }
}

impl TypeSystemId for ValueId {
    type Reader<'a> = Value<'a>;

    fn read(self, ast: &super::TypeSystemDocument) -> Self::Reader<'_> {
        match ast.lookup(self) {
            ValueRecord::Variable(id) => Value::Variable(ast.lookup(*id)),
            ValueRecord::Int(num) => Value::Int(IntValue(*num)),
            ValueRecord::Float(num) => Value::Float(*num),
            ValueRecord::String(id) => Value::String(ast.lookup(*id)),
            ValueRecord::BlockString(id) => Value::BlockString(ast.lookup(*id)),
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
