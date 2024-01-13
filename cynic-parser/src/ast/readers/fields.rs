use crate::ast::{ids::FieldDefinitionId, AstLookup};

use super::{
    directives::Directive, input_values::InputValueDefinition, types::Type, AstId, ReadContext,
};

#[derive(Clone, Copy)]
pub struct FieldDefinition<'a>(ReadContext<'a, FieldDefinitionId>);

impl<'a> FieldDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let ast = &self.0.ast;
        ast.lookup(ast.lookup(self.0.id).name)
    }

    pub fn ty(&self) -> Type<'a> {
        let ast = &self.0.ast;
        ast.read(ast.lookup(self.0.id).ty)
    }

    pub fn arguments(&self) -> impl Iterator<Item = InputValueDefinition<'a>> {
        let ast = &self.0.ast;
        ast.lookup(self.0.id)
            .arguments
            .iter()
            .map(|id| ast.read(*id))
    }

    pub fn directives(&self) -> impl Iterator<Item = Directive<'a>> + 'a {
        let ast = &self.0.ast;
        ast.lookup(self.0.id)
            .directives
            .iter()
            .map(|id| ast.read(*id))
    }
}

impl AstId for FieldDefinitionId {
    type Reader<'a> = FieldDefinition<'a>;
}

impl<'a> From<ReadContext<'a, FieldDefinitionId>> for FieldDefinition<'a> {
    fn from(value: ReadContext<'a, FieldDefinitionId>) -> Self {
        Self(value)
    }
}
