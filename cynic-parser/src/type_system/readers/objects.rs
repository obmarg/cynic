use crate::type_system::{ids::ObjectDefinitionId, AstLookup};

use super::{directives::Directive, fields::FieldDefinition, AstId, ReadContext};

#[derive(Clone, Copy)]
pub struct ObjectDefinition<'a>(ReadContext<'a, ObjectDefinitionId>);

impl<'a> ObjectDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let ast = &self.0.ast;

        ast.lookup(ast.lookup(self.0.id).name)
    }

    pub fn description(&self) -> Option<&'a str> {
        let ast = &self.0.ast;

        ast.lookup(self.0.id).description.map(|id| ast.lookup(id))
    }

    pub fn implements_interfaces(&self) -> impl Iterator<Item = &'a str> + 'a {
        let ast = &self.0.ast;

        ast.lookup(self.0.id)
            .implements
            .iter()
            .map(|id| ast.lookup(*id))
    }

    pub fn fields(&self) -> impl Iterator<Item = FieldDefinition<'a>> + 'a {
        let ast = &self.0.ast;

        ast.lookup(self.0.id).fields.iter().map(|id| ast.read(id))
    }

    pub fn directives(&self) -> impl Iterator<Item = Directive<'a>> + 'a {
        let ast = &self.0.ast;

        ast.lookup(self.0.id)
            .directives
            .iter()
            .map(|id| ast.read(id))
    }
}

impl AstId for ObjectDefinitionId {
    type Reader<'a> = ObjectDefinition<'a>;
}

impl<'a> From<ReadContext<'a, ObjectDefinitionId>> for ObjectDefinition<'a> {
    fn from(value: ReadContext<'a, ObjectDefinitionId>) -> Self {
        Self(value)
    }
}
