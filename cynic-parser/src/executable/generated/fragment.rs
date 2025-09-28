use super::prelude::*;
use super::{
    ExecutableId,
    directive::Directive,
    ids::{DirectiveId, FragmentDefinitionId, SelectionId},
    selections::Selection,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct FragmentDefinitionRecord {
    pub name: StringId,
    pub name_span: Span,
    pub type_condition: StringId,
    pub type_condition_span: Span,
    pub directives: IdRange<DirectiveId>,
    pub selection_set: IdRange<SelectionId>,
    pub selection_set_span: Span,
}

#[derive(Clone, Copy)]
pub struct FragmentDefinition<'a>(pub(in super::super) ReadContext<'a, FragmentDefinitionId>);

impl<'a> FragmentDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).name)
    }
    pub fn name_span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).name_span
    }
    pub fn type_condition(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).type_condition)
    }
    pub fn type_condition_span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).type_condition_span
    }
    pub fn directives(&self) -> Iter<'a, Directive<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).directives, document)
    }
    pub fn selection_set(&self) -> Iter<'a, Selection<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).selection_set, document)
    }
    pub fn selection_set_span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).selection_set_span
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

impl IdReader for FragmentDefinition<'_> {
    type Id = FragmentDefinitionId;
    type Reader<'a> = FragmentDefinition<'a>;
    fn new(id: Self::Id, document: &'_ ExecutableDocument) -> Self::Reader<'_> {
        document.read(id)
    }
}
