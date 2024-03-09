use crate::{
    type_system::{self, ids::ValueId},
    AstLookup,
};

use super::{ReadContext, TypeSystemId};

#[derive(Clone)]
pub enum ValueReader<'a> {
    Variable(&'a str),
    Int(i32),
    Float(f32),
    String(&'a str),
    Boolean(bool),
    Null,
    Enum(&'a str),
    List(Vec<ValueReader<'a>>),
    Object(Vec<(&'a str, ValueReader<'a>)>),
}

impl TypeSystemId for ValueId {
    type Reader<'a> = ValueReader<'a>;
}

impl<'a> From<ReadContext<'a, ValueId>> for ValueReader<'a> {
    fn from(reader: ReadContext<'a, ValueId>) -> Self {
        let ast = &reader.ast;

        match ast.lookup(reader.id) {
            type_system::Value::Variable(id) => ValueReader::Variable(ast.lookup(*id)),
            type_system::Value::Int(num) => ValueReader::Int(*num),
            type_system::Value::Float(num) => ValueReader::Float(*num),
            type_system::Value::String(id) => ValueReader::String(ast.lookup(*id)),
            type_system::Value::Boolean(val) => ValueReader::Boolean(*val),
            type_system::Value::Null => ValueReader::Null,
            type_system::Value::Enum(id) => ValueReader::Enum(ast.lookup(*id)),
            type_system::Value::List(ids) => {
                ValueReader::List(ids.iter().map(|id| ast.read(*id)).collect())
            }
            type_system::Value::Object(pairs) => ValueReader::Object(
                pairs
                    .iter()
                    .map(|(name, value)| (ast.lookup(*name), ast.read(*value)))
                    .collect(),
            ),
        }
    }
}
