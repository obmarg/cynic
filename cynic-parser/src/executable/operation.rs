use crate::common::{IdRange, OperationType};

use super::ids::{DirectiveId, SelectionId, StringId, VariableDefinitionId};

pub struct OperationDefinitionRecord {
    pub operation_type: OperationType,
    pub name: Option<StringId>,
    pub variable_definitions: IdRange<VariableDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub selection_set: IdRange<SelectionId>,
}
