use crate::{common::IdRange, AstLookup};

use super::{
    argument::Argument,
    directive::Directive,
    ids::{
        ArgumentId, DirectiveId, FieldSelectionId, FragmentSpreadId, InlineFragmentId, SelectionId,
        StringId,
    },
    ExecutableId, ReadContext,
};

pub enum SelectionRecord {
    Field(FieldSelectionId),
    InlineFragment(InlineFragmentId),
    FragmentSpread(FragmentSpreadId),
}

pub struct FieldSelectionRecord {
    pub alias: Option<StringId>,
    pub name: StringId,
    pub arguments: IdRange<ArgumentId>,
    pub directives: IdRange<DirectiveId>,
    pub selection_set: IdRange<SelectionId>,
}

pub struct InlineFragmentRecord {
    pub type_condition: Option<StringId>,
    pub directives: IdRange<DirectiveId>,
    pub selection_set: IdRange<SelectionId>,
}

pub struct FragmentSpreadRecord {
    pub name: StringId,
    pub directives: IdRange<DirectiveId>,
}

pub enum Selection<'a> {
    Field(FieldSelection<'a>),
    InlineFragment(InlineFragment<'a>),
    FragmentSpread(FragmentSpread<'a>),
}

impl ExecutableId for SelectionId {
    type Reader<'a> = Selection<'a>;
}

impl<'a> From<ReadContext<'a, SelectionId>> for Selection<'a> {
    fn from(value: ReadContext<'a, SelectionId>) -> Self {
        match value.ast.lookup(value.id) {
            SelectionRecord::Field(id) => Selection::Field(value.ast.read(*id)),
            SelectionRecord::InlineFragment(id) => Selection::InlineFragment(value.ast.read(*id)),
            SelectionRecord::FragmentSpread(id) => Selection::FragmentSpread(value.ast.read(*id)),
        }
    }
}

#[derive(Clone, Copy)]
pub struct FieldSelection<'a>(ReadContext<'a, FieldSelectionId>);

impl<'a> FieldSelection<'a> {
    pub fn alias(&self) -> Option<&'a str> {
        self.0
            .ast
            .lookup(self.0.id)
            .alias
            .map(|id| self.0.ast.lookup(id))
    }

    pub fn name(&self) -> &'a str {
        self.0.ast.lookup(self.0.ast.lookup(self.0.id).name)
    }

    pub fn arguments(&self) -> impl Iterator<Item = Argument<'a>> {
        let ast = &self.0.ast;

        ast.lookup(self.0.id)
            .arguments
            .iter()
            .map(|id| ast.read(id))
    }

    pub fn directives(&self) -> impl Iterator<Item = Directive<'a>> {
        self.0
            .ast
            .lookup(self.0.id)
            .directives
            .iter()
            .map(|id| self.0.ast.read(id))
    }

    pub fn selection_set(&self) -> impl Iterator<Item = Selection<'a>> {
        self.0
            .ast
            .lookup(self.0.id)
            .selection_set
            .iter()
            .map(|id| self.0.ast.read(id))
    }
}

impl ExecutableId for FieldSelectionId {
    type Reader<'a> = FieldSelection<'a>;
}

impl<'a> From<ReadContext<'a, FieldSelectionId>> for FieldSelection<'a> {
    fn from(value: ReadContext<'a, FieldSelectionId>) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy)]
pub struct InlineFragment<'a>(ReadContext<'a, InlineFragmentId>);

impl<'a> InlineFragment<'a> {}

impl ExecutableId for InlineFragmentId {
    type Reader<'a> = InlineFragment<'a>;
}

impl<'a> From<ReadContext<'a, InlineFragmentId>> for InlineFragment<'a> {
    fn from(value: ReadContext<'a, InlineFragmentId>) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy)]
pub struct FragmentSpread<'a>(ReadContext<'a, FragmentSpreadId>);

impl<'a> FragmentSpread<'a> {}

impl ExecutableId for FragmentSpreadId {
    type Reader<'a> = FragmentSpread<'a>;
}

impl<'a> From<ReadContext<'a, FragmentSpreadId>> for FragmentSpread<'a> {
    fn from(value: ReadContext<'a, FragmentSpreadId>) -> Self {
        Self(value)
    }
}
