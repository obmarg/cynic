use super::prelude::*;
use super::{
    directive::Directive,
    ids::{DirectiveId, FragmentDefinitionId, SelectionId},
    selections::Selection,
    ExecutableId,
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
pub struct FragmentDefinition<'a>(pub(in super::super) ReadContext<'a, FragmentDefinitionId>);

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

impl FragmentDefinition<'_> {
    pub fn id(&self) -> FragmentDefinitionId {
        self.0.id
    }
}

impl fmt::Debug for FragmentDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FragmentDefinition")
            .field("name", &self.name())
            .field("type_condition", &self.type_condition())
            .field("directives", &self.directives())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

impl ExecutableId for FragmentDefinitionId {
    type Reader<'a> = FragmentDefinition<'a>;
    fn read(self, document: &ExecutableDocument) -> Self::Reader<'_> {
        FragmentDefinition(ReadContext { id: self, document })
    }
}

impl<'a> IdReader<'a> for FragmentDefinition<'a> {
    type Id = FragmentDefinitionId;
    fn new(id: Self::Id, document: &'a ExecutableDocument) -> Self {
        document.read(id)
    }
}
