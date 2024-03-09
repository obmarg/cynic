use crate::{common::IdRange, AstLookup};

use super::{
    directive::Directive,
    ids::{DirectiveId, FragmentDefinitionId, SelectionId, StringId},
    selections::Selection,
    ExecutableId, ReadContext,
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
        let ast = &self.0.ast;
        ast.lookup(ast.lookup(self.0.id).name)
    }

    pub fn type_condition(&self) -> &'a str {
        let ast = &self.0.ast;
        ast.lookup(ast.lookup(self.0.id).type_condition)
    }

    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> {
        self.0
            .ast
            .lookup(self.0.id)
            .directives
            .iter()
            .map(|id| self.0.ast.read(id))
    }

    pub fn selection_set(&self) -> impl ExactSizeIterator<Item = Selection<'a>> {
        self.0
            .ast
            .lookup(self.0.id)
            .selection_set
            .iter()
            .map(|id| self.0.ast.read(id))
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
