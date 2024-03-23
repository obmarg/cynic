use crate::{
    type_system::{self, ids::ValueId},
    AstLookup,
};

use super::{ReadContext, TypeSystemId};

#[derive(Clone)]
pub enum Value<'a> {
    Variable(&'a str),
    Int(i32),
    Float(f32),
    String(&'a str),
    BlockString(&'a str),
    Boolean(bool),
    Null,
    Enum(&'a str),
    List(Vec<Value<'a>>),
    Object(Vec<(&'a str, Value<'a>)>),
}

impl<'a> Value<'a> {
    pub fn as_str(&self) -> Option<&'a str> {
        match self {
            Self::String(inner) => Some(inner),
            _ => None,
        }
    }
}

impl TypeSystemId for ValueId {
    type Reader<'a> = Value<'a>;
}

impl<'a> From<ReadContext<'a, ValueId>> for Value<'a> {
    fn from(reader: ReadContext<'a, ValueId>) -> Self {
        let ast = &reader.document;

        match ast.lookup(reader.id) {
            type_system::ValueRecord::Variable(id) => Value::Variable(ast.lookup(*id)),
            type_system::ValueRecord::Int(num) => Value::Int(*num),
            type_system::ValueRecord::Float(num) => Value::Float(*num),
            type_system::ValueRecord::String(id) => Value::String(ast.lookup(*id)),
            type_system::ValueRecord::BlockString(id) => Value::BlockString(ast.lookup(*id)),
            type_system::ValueRecord::Boolean(val) => Value::Boolean(*val),
            type_system::ValueRecord::Null => Value::Null,
            type_system::ValueRecord::Enum(id) => Value::Enum(ast.lookup(*id)),
            type_system::ValueRecord::List(ids) => {
                Value::List(ids.iter().map(|id| ast.read(*id)).collect())
            }
            type_system::ValueRecord::Object(pairs) => Value::Object(
                pairs
                    .iter()
                    .map(|(name, value)| (ast.lookup(*name), ast.read(*value)))
                    .collect(),
            ),
        }
    }
}
