use crate::{type_system::ids::InputValueDefinitionId, AstLookup};

use super::{directives::Directive, types::Type, values::ValueReader, ReadContext, TypeSystemId};

#[derive(Clone, Copy)]
pub struct InputValueDefinition<'a>(ReadContext<'a, InputValueDefinitionId>);

impl<'a> InputValueDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let ast = &self.0.ast;
        ast.lookup(ast.lookup(self.0.id).name)
    }

    pub fn ty(&self) -> Type<'a> {
        let ast = &self.0.ast;

        ast.read(ast.lookup(self.0.id).ty)
    }

    pub fn description(&self) -> Option<&'a str> {
        let ast = &self.0.ast;
        ast.lookup(self.0.id).description.map(|id| ast.lookup(id))
    }

    pub fn default_value(&self) -> Option<ValueReader<'a>> {
        let ast = &self.0.ast;
        ast.lookup(self.0.id).default.map(|id| ast.read(id))
    }

    pub fn directives(&self) -> impl Iterator<Item = Directive<'a>> + 'a {
        let ast = &self.0.ast;
        ast.lookup(self.0.id)
            .directives
            .iter()
            .map(|id| ast.read(id))
    }
}

impl TypeSystemId for InputValueDefinitionId {
    type Reader<'a> = InputValueDefinition<'a>;
}

impl<'a> From<ReadContext<'a, InputValueDefinitionId>> for InputValueDefinition<'a> {
    fn from(value: ReadContext<'a, InputValueDefinitionId>) -> Self {
        Self(value)
    }
}
