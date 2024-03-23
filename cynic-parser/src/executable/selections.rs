#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    argument::Argument,
    directive::Directive,
    ids::{
        ArgumentId, DirectiveId, FieldSelectionId, FragmentSpreadId, InlineFragmentId, SelectionId,
    },
    ExecutableId, ReadContext,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    AstLookup,
};

pub enum SelectionRecord {
    FieldSelection(FieldSelectionId),
    InlineFragment(InlineFragmentId),
    FragmentSpread(FragmentSpreadId),
}

#[derive(Clone, Copy)]
pub enum Selection<'a> {
    FieldSelection(FieldSelection<'a>),
    InlineFragment(InlineFragment<'a>),
    FragmentSpread(FragmentSpread<'a>),
}

impl ExecutableId for SelectionId {
    type Reader<'a> = Selection<'a>;
}

impl<'a> From<ReadContext<'a, SelectionId>> for Selection<'a> {
    fn from(value: ReadContext<'a, SelectionId>) -> Self {
        match value.document.lookup(value.id) {
            SelectionRecord::FieldSelection(id) => {
                Selection::FieldSelection(value.document.read(*id))
            }
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
pub struct FieldSelection<'a>(ReadContext<'a, FieldSelectionId>);

impl<'a> FieldSelection<'a> {
    pub fn alias(&self) -> Option<&'a str> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .alias
            .map(|id| document.lookup(id))
    }
    pub fn name(&self) -> &'a str {
        let ast = &self.0.document;
        ast.lookup(ast.lookup(self.0.id).name)
    }
    pub fn arguments(&self) -> impl ExactSizeIterator<Item = Argument<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .arguments
            .iter()
            .map(|id| document.read(id))
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

impl ExecutableId for FieldSelectionId {
    type Reader<'a> = FieldSelection<'a>;
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
pub struct InlineFragment<'a>(ReadContext<'a, InlineFragmentId>);

impl<'a> InlineFragment<'a> {
    pub fn type_condition(&self) -> Option<&'a str> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .type_condition
            .map(|id| document.lookup(id))
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

impl ExecutableId for InlineFragmentId {
    type Reader<'a> = InlineFragment<'a>;
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
pub struct FragmentSpread<'a>(ReadContext<'a, FragmentSpreadId>);

impl<'a> FragmentSpread<'a> {
    pub fn fragment_name(&self) -> &'a str {
        let ast = &self.0.document;
        ast.lookup(ast.lookup(self.0.id).fragment_name)
    }
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .directives
            .iter()
            .map(|id| document.read(id))
    }
}

impl ExecutableId for FragmentSpreadId {
    type Reader<'a> = FragmentSpread<'a>;
}

impl<'a> From<ReadContext<'a, FragmentSpreadId>> for FragmentSpread<'a> {
    fn from(value: ReadContext<'a, FragmentSpreadId>) -> Self {
        Self(value)
    }
}
