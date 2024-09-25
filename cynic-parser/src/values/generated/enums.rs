use super::prelude::*;
use super::{ids::EnumValueId, ReadContext, ValueId};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct EnumValueRecord {
    pub value: StringId,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct EnumValue<'a>(pub(in super::super) ReadContext<'a, EnumValueId>);

impl<'a> EnumValue<'a> {
    pub fn value(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).value)
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl EnumValue<'_> {
    pub fn id(&self) -> EnumValueId {
        self.0.id
    }
}

impl fmt::Debug for EnumValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnumValue")
            .field("value", &self.value())
            .field("span", &self.span())
            .finish()
    }
}

impl ValueId for EnumValueId {
    type Reader<'a> = EnumValue<'a>;
}

impl IdReader for EnumValue<'_> {
    type Id = EnumValueId;
}

impl<'a> From<ReadContext<'a, EnumValueId>> for EnumValue<'a> {
    fn from(value: ReadContext<'a, EnumValueId>) -> Self {
        Self(value)
    }
}
