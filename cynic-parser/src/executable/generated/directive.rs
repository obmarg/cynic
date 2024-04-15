#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    argument::Argument,
    ids::{ArgumentId, DirectiveId},
    ExecutableId, ReadContext,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    AstLookup,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct DirectiveRecord {
    pub name: StringId,
    pub arguments: IdRange<ArgumentId>,
}

#[derive(Clone, Copy)]
pub struct Directive<'a>(ReadContext<'a, DirectiveId>);

impl<'a> Directive<'a> {
    pub fn name(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).name)
    }
    pub fn arguments(&self) -> impl ExactSizeIterator<Item = Argument<'a>> + 'a {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .arguments
            .iter()
            .map(|id| document.read(id))
    }
}

impl fmt::Debug for Directive<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Directive")
            .field("name", &self.name())
            .field("arguments", &self.arguments().collect::<Vec<_>>())
            .finish()
    }
}

impl ExecutableId for DirectiveId {
    type Reader<'a> = Directive<'a>;
}

impl<'a> From<ReadContext<'a, DirectiveId>> for Directive<'a> {
    fn from(value: ReadContext<'a, DirectiveId>) -> Self {
        Self(value)
    }
}
