use crate::common::IdRange;

use super::ids::{DirectiveId, SelectionId, StringId};

pub struct FragmentDefinitionRecord {
    pub name: StringId,
    pub type_condition: StringId,
    pub directives: IdRange<DirectiveId>,
    pub selection_set: IdRange<SelectionId>,
}
