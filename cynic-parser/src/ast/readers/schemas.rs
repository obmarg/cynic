use crate::ast::{
    ids::{AstLookup, SchemaDefinitionId},
    OperationType,
};

use super::{AstId, AstReader};

pub struct SchemaDefinition<'a>(AstReader<'a, SchemaDefinitionId>);

impl<'a> SchemaDefinition<'a> {
    pub fn description(&self) -> Option<&str> {
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

impl<'a> From<AstReader<'a, SchemaDefinitionId>> for SchemaDefinition<'a> {
    fn from(value: AstReader<'a, SchemaDefinitionId>) -> Self {
        Self(value)
    }
}
