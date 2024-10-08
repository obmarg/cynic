use super::prelude::*;
use super::{ids::NameId, ExecutableId};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct NameRecord {
    pub text: StringId,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct Name<'a>(pub(in super::super) ReadContext<'a, NameId>);

impl<'a> Name<'a> {
    pub fn text(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).text)
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
            .field("text", &self.text())
            .field("span", &self.span())
            .finish()
    }
}

impl ExecutableId for NameId {
    type Reader<'a> = Name<'a>;
    fn read(self, document: &ExecutableDocument) -> Self::Reader<'_> {
        Name(ReadContext { id: self, document })
    }
}

impl IdReader for Name<'_> {
    type Id = NameId;
}