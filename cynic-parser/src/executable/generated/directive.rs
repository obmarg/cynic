use super::prelude::*;
use super::{
    argument::Argument,
    ids::{ArgumentId, DirectiveId, NameId},
    name::Name,
    ExecutableId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct DirectiveRecord {
    pub name: NameId,
    pub arguments: IdRange<ArgumentId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct Directive<'a>(pub(in super::super) ReadContext<'a, DirectiveId>);

impl<'a> Directive<'a> {
    pub fn name(&self) -> Name<'a> {
        let document = self.0.document;
        document.read(document.lookup(self.0.id).name)
    }
    pub fn arguments(&self) -> Iter<'a, Argument<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).arguments, document)
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl Directive<'_> {
    pub fn id(&self) -> DirectiveId {
        self.0.id
    }
}

impl fmt::Debug for Directive<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Directive")
            .field("name", &self.name())
            .field("arguments", &self.arguments())
            .field("span", &self.span())
            .finish()
    }
}

impl ExecutableId for DirectiveId {
    type Reader<'a> = Directive<'a>;
    fn read(self, document: &ExecutableDocument) -> Self::Reader<'_> {
        Directive(ReadContext { id: self, document })
    }
}

impl IdReader for Directive<'_> {
    type Id = DirectiveId;
}
