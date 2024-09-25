use super::prelude::*;
use super::{
    ids::{DescriptionId, StringLiteralId},
    strings::StringLiteral,
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct DescriptionRecord {
    pub description: StringLiteralId,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct Description<'a>(pub(in super::super) ReadContext<'a, DescriptionId>);

impl<'a> Description<'a> {
    pub fn description(&self) -> StringLiteral<'a> {
        let document = self.0.document;
        document.read(document.lookup(self.0.id).description)
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl Description<'_> {
    pub fn id(&self) -> DescriptionId {
        self.0.id
    }
}

impl fmt::Debug for Description<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Description")
            .field("description", &self.description())
            .field("span", &self.span())
            .finish()
    }
}

impl TypeSystemId for DescriptionId {
    type Reader<'a> = Description<'a>;
}

impl IdReader for Description<'_> {
    type Id = DescriptionId;
}

impl<'a> From<ReadContext<'a, DescriptionId>> for Description<'a> {
    fn from(value: ReadContext<'a, DescriptionId>) -> Self {
        Self(value)
    }
}
