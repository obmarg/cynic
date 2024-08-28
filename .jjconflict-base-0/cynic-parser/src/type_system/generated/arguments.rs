use super::prelude::*;
use super::{
    ids::{ArgumentId, ValueId},
    value::Value,
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct ArgumentRecord {
    pub name: StringId,
    pub value: ValueId,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct Argument<'a>(pub(in super::super) ReadContext<'a, ArgumentId>);

impl<'a> Argument<'a> {
    pub fn name(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).name)
    }
    pub fn value(&self) -> Value<'a> {
        let document = self.0.document;
        document.read(document.lookup(self.0.id).value)
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl Argument<'_> {
    pub fn id(&self) -> ArgumentId {
        self.0.id
    }
}

impl fmt::Debug for Argument<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Argument")
            .field("name", &self.name())
            .field("value", &self.value())
            .field("span", &self.span())
            .finish()
    }
}

impl TypeSystemId for ArgumentId {
    type Reader<'a> = Argument<'a>;
}

impl IdReader for Argument<'_> {
    type Id = ArgumentId;
}

impl<'a> From<ReadContext<'a, ArgumentId>> for Argument<'a> {
    fn from(value: ReadContext<'a, ArgumentId>) -> Self {
        Self(value)
    }
}
