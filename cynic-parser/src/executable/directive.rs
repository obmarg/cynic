use crate::{common::IdRange, AstLookup};

use super::{
    argument::Argument,
    ids::{ArgumentId, DirectiveId, StringId},
    ExecutableId, ReadContext,
};

pub struct DirectiveRecord {
    pub name: StringId,
    pub arguments: IdRange<ArgumentId>,
}

#[derive(Clone, Copy)]
pub struct Directive<'a>(ReadContext<'a, DirectiveId>);

impl<'a> Directive<'a> {
    pub fn name(&self) -> &'a str {
        let ast = &self.0.document;

        ast.lookup(ast.lookup(self.0.id).name)
    }

    pub fn arguments(&self) -> impl ExactSizeIterator<Item = Argument<'a>> {
        let ast = &self.0.document;

        ast.lookup(self.0.id)
            .arguments
            .iter()
            .map(|id| ast.read(id))
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
