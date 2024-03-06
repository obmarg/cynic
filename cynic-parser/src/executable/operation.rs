pub struct OperationDefinitionRecord {
    operation_type: OperationType,
    name: StringId,
    variables: IdRange<VariableId>,
    directives: IdRange<DirectiveId>,
    selection_set: SelectionSetId,
}
