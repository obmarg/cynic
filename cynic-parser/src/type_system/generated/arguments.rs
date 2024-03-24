#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    ids::{ArgumentId, ValueId},
    value::Value,
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    type_system::DirectiveLocation,
    AstLookup, Span,
};

pub struct ArgumentRecord {
    pub name: StringId,
    pub value: ValueId,
}

#[derive(Clone, Copy)]
pub struct Argument<'a>(ReadContext<'a, ArgumentId>);

impl<'a> Argument<'a> {
    pub fn name(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).name)
    }
    pub fn value(&self) -> Value<'a> {
        let document = self.0.document;
        document.read(document.lookup(self.0.id).value)
    }
}

impl TypeSystemId for ArgumentId {
    type Reader<'a> = Argument<'a>;
}

impl<'a> From<ReadContext<'a, ArgumentId>> for Argument<'a> {
    fn from(value: ReadContext<'a, ArgumentId>) -> Self {
        Self(value)
    }
}
