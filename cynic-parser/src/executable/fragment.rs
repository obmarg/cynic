use crate::common::{IdRange, OperationType};

use super::ids::{SelectionId, StringId};

pub struct FragmentDefinitionRecord {
    pub operation_type: OperationType,
    pub name: StringId,
    // type_condition: TypeId
    // directives: IdRange<DirectiveId>,
    pub selection_set: IdRange<SelectionId>,
}
