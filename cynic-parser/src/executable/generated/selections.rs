use super::prelude::*;
use super::{
    argument::Argument,
    directive::Directive,
    ids::{
        ArgumentId, DirectiveId, FieldSelectionId, FragmentSpreadId, InlineFragmentId, SelectionId,
    },
    ExecutableId, ReadContext,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub enum SelectionRecord {
    Field(FieldSelectionId),
    InlineFragment(InlineFragmentId),
    FragmentSpread(FragmentSpreadId),
}

#[derive(Clone, Copy, Debug)]
pub enum Selection<'a> {
    Field(FieldSelection<'a>),
    InlineFragment(InlineFragment<'a>),
    FragmentSpread(FragmentSpread<'a>),
}

impl ExecutableId for SelectionId {
    type Reader<'a> = Selection<'a>;
}

impl IdReader for Selection<'_> {
    type Id = SelectionId;
}

impl<'a> From<ReadContext<'a, SelectionId>> for Selection<'a> {
    fn from(value: ReadContext<'a, SelectionId>) -> Self {
        match value.document.lookup(value.id) {
            SelectionRecord::Field(id) => Selection::Field(value.document.read(*id)),
            SelectionRecord::InlineFragment(id) => {
                Selection::InlineFragment(value.document.read(*id))
            }
            SelectionRecord::FragmentSpread(id) => {
                Selection::FragmentSpread(value.document.read(*id))
            }
        }
    }
}

pub struct FieldSelectionRecord {
    pub alias: Option<StringId>,
    pub name: StringId,
    pub arguments: IdRange<ArgumentId>,
    pub directives: IdRange<DirectiveId>,
    pub selection_set: IdRange<SelectionId>,
}

#[derive(Clone, Copy)]
pub struct FieldSelection<'a>(pub(in super::super) ReadContext<'a, FieldSelectionId>);

impl<'a> FieldSelection<'a> {
    pub fn alias(&self) -> Option<&'a str> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .alias
            .map(|id| document.lookup(id))
    }
    pub fn name(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).name)
    }
    pub fn arguments(&self) -> Iter<'a, Argument<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).arguments, document)
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

impl FieldSelection<'_> {
    pub fn id(&self) -> FieldSelectionId {
        self.0.id
    }
}

impl fmt::Debug for FieldSelection<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FieldSelection")
            .field("alias", &self.alias())
            .field("name", &self.name())
            .field("arguments", &self.arguments())
            .field("directives", &self.directives())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

impl ExecutableId for FieldSelectionId {
    type Reader<'a> = FieldSelection<'a>;
}

impl IdReader for FieldSelection<'_> {
    type Id = FieldSelectionId;
}

impl<'a> From<ReadContext<'a, FieldSelectionId>> for FieldSelection<'a> {
    fn from(value: ReadContext<'a, FieldSelectionId>) -> Self {
        Self(value)
    }
}

pub struct InlineFragmentRecord {
    pub type_condition: Option<StringId>,
    pub directives: IdRange<DirectiveId>,
    pub selection_set: IdRange<SelectionId>,
}

#[derive(Clone, Copy)]
pub struct InlineFragment<'a>(pub(in super::super) ReadContext<'a, InlineFragmentId>);

impl<'a> InlineFragment<'a> {
    pub fn type_condition(&self) -> Option<&'a str> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .type_condition
            .map(|id| document.lookup(id))
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

impl InlineFragment<'_> {
    pub fn id(&self) -> InlineFragmentId {
        self.0.id
    }
}

impl fmt::Debug for InlineFragment<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InlineFragment")
            .field("type_condition", &self.type_condition())
            .field("directives", &self.directives())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

impl ExecutableId for InlineFragmentId {
    type Reader<'a> = InlineFragment<'a>;
}

impl IdReader for InlineFragment<'_> {
    type Id = InlineFragmentId;
}

impl<'a> From<ReadContext<'a, InlineFragmentId>> for InlineFragment<'a> {
    fn from(value: ReadContext<'a, InlineFragmentId>) -> Self {
        Self(value)
    }
}

pub struct FragmentSpreadRecord {
    pub fragment_name: StringId,
    pub directives: IdRange<DirectiveId>,
}

#[derive(Clone, Copy)]
pub struct FragmentSpread<'a>(pub(in super::super) ReadContext<'a, FragmentSpreadId>);

impl<'a> FragmentSpread<'a> {
    pub fn fragment_name(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).fragment_name)
    }
    pub fn directives(&self) -> Iter<'a, Directive<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).directives, document)
    }
}

impl FragmentSpread<'_> {
    pub fn id(&self) -> FragmentSpreadId {
        self.0.id
    }
}

impl fmt::Debug for FragmentSpread<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FragmentSpread")
            .field("fragment_name", &self.fragment_name())
            .field("directives", &self.directives())
            .finish()
    }
}

impl ExecutableId for FragmentSpreadId {
    type Reader<'a> = FragmentSpread<'a>;
}

impl IdReader for FragmentSpread<'_> {
    type Id = FragmentSpreadId;
}

impl<'a> From<ReadContext<'a, FragmentSpreadId>> for FragmentSpread<'a> {
    fn from(value: ReadContext<'a, FragmentSpreadId>) -> Self {
        Self(value)
    }
}
