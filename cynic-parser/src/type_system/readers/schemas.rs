use crate::{common::OperationType, type_system::ids::SchemaDefinitionId, AstLookup};

use super::{ReadContext, TypeSystemId};

#[derive(Clone, Copy)]
pub struct SchemaDefinition<'a>(ReadContext<'a, SchemaDefinitionId>);

impl<'a> SchemaDefinition<'a> {
    pub fn description(&self) -> Option<&'a str> {
        let ast = &self.0.document;

        ast.lookup(self.0.id).description.map(|id| ast.lookup(id))
    }

    pub fn query_type(&self) -> Option<&'a str> {
        self.root_operations()
            .find(|(ty, _)| matches!(ty, OperationType::Query))
            .map(|(_, name)| name)
    }

    pub fn mutation_type(&self) -> Option<&'a str> {
        self.root_operations()
            .find(|(ty, _)| matches!(ty, OperationType::Mutation))
            .map(|(_, name)| name)
    }

    pub fn subscription_type(&self) -> Option<&'a str> {
        self.root_operations()
            .find(|(ty, _)| matches!(ty, OperationType::Subscription))
            .map(|(_, name)| name)
    }

    pub fn root_operations(&self) -> impl ExactSizeIterator<Item = (OperationType, &'a str)> {
        let ast = &self.0.document;

        ast.lookup(self.0.id)
            .roots
            .iter()
            .map(|root| (root.operation_type, ast.lookup(root.named_type)))
    }
}

impl TypeSystemId for SchemaDefinitionId {
    type Reader<'a> = SchemaDefinition<'a>;
}

impl<'a> From<ReadContext<'a, SchemaDefinitionId>> for SchemaDefinition<'a> {
    fn from(value: ReadContext<'a, SchemaDefinitionId>) -> Self {
        Self(value)
    }
}
