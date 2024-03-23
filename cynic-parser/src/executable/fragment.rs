#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    directive::Directive,
    ids::{DirectiveId, FragmentDefinitionId, SelectionId},
    selections::Selection,
    ExecutableId, ReadContext,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    AstLookup,
};

pub struct FragmentDefinitionRecord {
    pub name: StringId,
    pub type_condition: StringId,
    pub directives: IdRange<DirectiveId>,
    pub selection_set: IdRange<SelectionId>,
}

#[derive(Clone, Copy)]
pub struct FragmentDefinition<'a>(ReadContext<'a, FragmentDefinitionId>);

impl<'a> FragmentDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let ast = &self.0.document;
        ast.lookup(ast.lookup(self.0.id).name)
    }
    pub fn type_condition(&self) -> &'a str {
        let ast = &self.0.document;
        ast.lookup(ast.lookup(self.0.id).type_condition)
    }
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .directives
            .iter()
            .map(|id| document.read(id))
    }
    pub fn selection_set(&self) -> impl ExactSizeIterator<Item = Selection<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .selection_set
            .iter()
            .map(|id| document.read(id))
    }
}

impl ExecutableId for FragmentDefinitionId {
    type Reader<'a> = FragmentDefinition<'a>;
}

impl<'a> From<ReadContext<'a, FragmentDefinitionId>> for FragmentDefinition<'a> {
    fn from(value: ReadContext<'a, FragmentDefinitionId>) -> Self {
        Self(value)
    }
}

