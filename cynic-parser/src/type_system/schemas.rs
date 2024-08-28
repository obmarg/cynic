use crate::common::OperationType;

impl<'a> super::SchemaDefinition<'a> {
    pub fn query_type(&self) -> Option<super::RootOperationTypeDefinition<'a>> {
        self.root_operations()
            .find(|operation| matches!(operation.operation_type(), OperationType::Query))
    }

    pub fn mutation_type(&self) -> Option<super::RootOperationTypeDefinition<'a>> {
        self.root_operations()
            .find(|operation| matches!(operation.operation_type(), OperationType::Mutation))
    }

    pub fn subscription_type(&self) -> Option<super::RootOperationTypeDefinition<'a>> {
        self.root_operations()
            .find(|operation| matches!(operation.operation_type(), OperationType::Subscription))
    }
}
