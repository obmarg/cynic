use super::prelude::*;
use super::{ids::NameId, ReadContext, ValueId};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct NameRecord {
    pub value: StringId,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct Name<'a>(pub(in super::super) ReadContext<'a, NameId>);

impl<'a> Name<'a> {
    pub fn value(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).value)
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl Name<'_> {
    pub fn id(&self) -> NameId {
        self.0.id
    }
}

impl fmt::Debug for Name<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Name")
            .field("value", &self.value())
            .field("span", &self.span())
            .finish()
    }
}

impl ValueId for NameId {
    type Reader<'a> = Name<'a>;
}

impl IdReader for Name<'_> {
    type Id = NameId;
}

impl<'a> From<ReadContext<'a, NameId>> for Name<'a> {
    fn from(value: ReadContext<'a, NameId>) -> Self {
        Self(value)
    }
}