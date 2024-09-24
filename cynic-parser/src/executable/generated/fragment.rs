use super::prelude::*;
use super::{
    directive::Directive,
    ids::{DirectiveId, FragmentDefinitionId, NameId, SelectionId, TypeConditionId},
    name::Name,
    selections::Selection,
    type_conditions::TypeCondition,
    ExecutableId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct FragmentDefinitionRecord {
    pub name: NameId,
    pub type_condition: TypeConditionId,
    pub directives: IdRange<DirectiveId>,
    pub selection_set: IdRange<SelectionId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct FragmentDefinition<'a>(pub(in super::super) ReadContext<'a, FragmentDefinitionId>);

impl<'a> FragmentDefinition<'a> {
    pub fn name(&self) -> Name<'a> {
        let document = self.0.document;
        document.read(document.lookup(self.0.id).name)
    }
    pub fn type_condition(&self) -> TypeCondition<'a> {
        let document = self.0.document;
        document.read(document.lookup(self.0.id).type_condition)
    }
    pub fn directives(&self) -> Iter<'a, Directive<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).directives, document)
    }
    pub fn selection_set(&self) -> Iter<'a, Selection<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).selection_set, document)
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
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
            .field("span", &self.span())
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
}
