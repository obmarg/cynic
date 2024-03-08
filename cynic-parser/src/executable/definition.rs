use super::ids::{FragmentDefinitionId, OperationDefinitionId};

pub enum ExecutableDefinitionRecord {
    Operation(OperationDefinitionId),
    Fragment(FragmentDefinitionId),
}
