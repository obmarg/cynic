use super::prelude::*;
use super::{
    directive::Directive,
    ids::{DirectiveId, FragmentDefinitionId, SelectionId},
    selections::Selection,
    ExecutableId, ReadContext,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

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
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).name)
    }
    pub fn type_condition(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).type_condition)
    }
    pub fn directives(&self) -> Iter<'a, Directive<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).directives, document)
    }
    pub fn selection_set(&self) -> Iter<'a, Selection<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).selection_set, document)
    }
}

impl fmt::Debug for FragmentDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FragmentDefinition")
            .field("name", &self.name())
            .field("type_condition", &self.type_condition())
            .field("directives", &self.directives().collect::<Vec<_>>())
            .field("selection_set", &self.selection_set().collect::<Vec<_>>())
            .finish()
    }
}

impl ExecutableId for FragmentDefinitionId {
    type Reader<'a> = FragmentDefinition<'a>;
}

impl super::IdReader for FragmentDefinition<'_> {
    type Id = FragmentDefinitionId;
}

impl<'a> From<ReadContext<'a, FragmentDefinitionId>> for FragmentDefinition<'a> {
    fn from(value: ReadContext<'a, FragmentDefinitionId>) -> Self {
        Self(value)
    }
}