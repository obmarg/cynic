use crate::ast::{ids::SchemaDefinitionId, AstLookup, OperationType};

use super::{AstId, ReadContext};

#[derive(Clone, Copy)]
pub struct SchemaDefinition<'a>(ReadContext<'a, SchemaDefinitionId>);

impl<'a> SchemaDefinition<'a> {
    pub fn description(&self) -> Option<&'a str> {
        let ast = &self.0.ast;

        ast.lookup(self.0.id).description.map(|id| ast.lookup(id))
    }

    pub fn root_operations(&self) -> impl Iterator<Item = (OperationType, &'a str)> {
        let ast = &self.0.ast;

        ast.lookup(self.0.id)
            .roots
            .iter()
            .map(|root| (root.operation_type, ast.lookup(root.named_type)))
    }
}

impl AstId for SchemaDefinitionId {
    type Reader<'a> = SchemaDefinition<'a>;
}

impl<'a> From<ReadContext<'a, SchemaDefinitionId>> for SchemaDefinition<'a> {
    fn from(value: ReadContext<'a, SchemaDefinitionId>) -> Self {
        Self(value)
    }
}
