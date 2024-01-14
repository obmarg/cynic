use crate::ast::{self, ids::ValueId, AstLookup};

use super::{AstId, ReadContext};

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

impl AstId for ValueId {
    type Reader<'a> = ValueReader<'a>;
}

impl<'a> From<ReadContext<'a, ValueId>> for ValueReader<'a> {
    fn from(reader: ReadContext<'a, ValueId>) -> Self {
        let ast = &reader.ast;

        match ast.lookup(reader.id) {
            ast::Value::Variable(id) => ValueReader::Variable(ast.lookup(*id)),
            ast::Value::Int(num) => ValueReader::Int(*num),
            ast::Value::Float(num) => ValueReader::Float(*num),
            ast::Value::String(id) => ValueReader::String(ast.lookup(*id)),
            ast::Value::Boolean(val) => ValueReader::Boolean(*val),
            ast::Value::Null => ValueReader::Null,
            ast::Value::Enum(id) => ValueReader::Enum(ast.lookup(*id)),
            ast::Value::List(ids) => {
                ValueReader::List(ids.iter().map(|id| ast.read(*id)).collect())
            }
            ast::Value::Object(pairs) => ValueReader::Object(
                pairs
                    .iter()
                    .map(|(name, value)| (ast.lookup(*name), ast.read(*value)))
                    .collect(),
            ),
        }
    }
}
