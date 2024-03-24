use crate::common::OperationType;

impl<'a> super::SchemaDefinition<'a> {
    pub fn query_type(&self) -> Option<&'a str> {
        self.root_operations()
            .find(|operation| matches!(operation.operation_type(), OperationType::Query))
            .map(|operation| operation.named_type())
    }

    pub fn mutation_type(&self) -> Option<&'a str> {
        self.root_operations()
            .find(|operation| matches!(operation.operation_type(), OperationType::Mutation))
            .map(|operation| operation.named_type())
    }

    pub fn subscription_type(&self) -> Option<&'a str> {
        self.root_operations()
            .find(|operation| matches!(operation.operation_type(), OperationType::Subscription))
            .map(|operation| operation.named_type())
    }
}
