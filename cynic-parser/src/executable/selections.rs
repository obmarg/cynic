use crate::common::IdRange;

use super::ids::{
    ArgumentId, DirectiveId, FieldSelectionId, FragmentSpreadId, InlineFragmentId, SelectionId,
    StringId,
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
