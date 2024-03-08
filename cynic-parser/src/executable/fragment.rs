use crate::common::OperationType;

use super::ids::StringId;

pub struct FragmentDefinitionRecord {
    operation_type: OperationType,
    name: StringId,
    // type_condition: TypeId
    // directives: IdRange<DirectiveId>,
    // selection_set: SelectionSetId,
}
