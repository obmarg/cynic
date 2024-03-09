use crate::common::{IdRange, OperationType};

use super::ids::{SelectionId, StringId};

pub struct OperationDefinitionRecord {
    pub operation_type: OperationType,
    pub name: Option<StringId>,
    // variables: IdRange<VariableId>,
    // directives: IdRange<DirectiveId>,
    pub selection_set: IdRange<SelectionId>,
}
