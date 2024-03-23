#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    ids::{ArgumentId, ValueId},
    value::Value,
    ExecutableId, ReadContext,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    AstLookup,
};

pub struct ArgumentRecord {
    pub name: StringId,
    pub value: ValueId,
}

#[derive(Clone, Copy)]
pub struct Argument<'a>(ReadContext<'a, ArgumentId>);

impl<'a> Argument<'a> {
    pub fn name(&self) -> &'a str {
        let ast = &self.0.document;
        ast.lookup(ast.lookup(self.0.id).name)
    }
    pub fn value(&self) -> Value<'a> {
        let document = self.0.document;
        document.read(document.lookup(self.0.id).value)
    }
}

impl ExecutableId for ArgumentId {
    type Reader<'a> = Argument<'a>;
}

impl<'a> From<ReadContext<'a, ArgumentId>> for Argument<'a> {
    fn from(value: ReadContext<'a, ArgumentId>) -> Self {
        Self(value)
    }
}
