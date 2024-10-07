use super::prelude::*;
use super::{
    ids::{DescriptionId, StringLiteralId},
    strings::StringLiteral,
    TypeSystemId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct DescriptionRecord {
    pub literal: StringLiteralId,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct Description<'a>(pub(in super::super) ReadContext<'a, DescriptionId>);

impl<'a> Description<'a> {
    pub fn literal(&self) -> StringLiteral<'a> {
        let document = self.0.document;
        document.read(document.lookup(self.0.id).literal)
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
            .field("literal", &self.literal())
            .field("span", &self.span())
            .finish()
    }
}

impl TypeSystemId for DescriptionId {
    type Reader<'a> = Description<'a>;
    fn read(self, document: &TypeSystemDocument) -> Self::Reader<'_> {
        Description(ReadContext { id: self, document })
    }
}

impl<'a> IdReader<'a> for Description<'a> {
    type Id = DescriptionId;
    fn new(id: Self::Id, document: &'a TypeSystemDocument) -> Self {
        document.read(id)
    }
}