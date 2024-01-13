use crate::ast::ids::{AstLookup, InterfaceDefinitionId};

use super::{directives::Directive, fields::FieldDefinition, AstId, ReadContext};

pub struct InterfaceDefinition<'a>(ReadContext<'a, InterfaceDefinitionId>);

impl<'a> InterfaceDefinition<'a> {
    pub fn name(&self) -> &str {
        self.0.ast.lookup(self.0.ast.lookup(self.0.id).name)
    }

    pub fn description(&self) -> Option<&str> {
        self.0
            .ast
            .lookup(self.0.id)
            .description
            .map(|id| self.0.ast.lookup(id))
    }

    pub fn implements_interfaces(&self) -> impl Iterator<Item = &'a str> + 'a {
        self.0
            .ast
            .lookup(self.0.id)
            .implements
            .iter()
            .map(|id| self.0.ast.lookup(*id))
    }

    pub fn fields(&self) -> impl Iterator<Item = FieldDefinition<'a>> + 'a {
        self.0
            .ast
            .lookup(self.0.id)
            .fields
            .iter()
            .map(|id| self.0.ast.read(*id))
    }

    pub fn directives(&self) -> impl Iterator<Item = Directive<'a>> + 'a {
        self.0
            .ast
            .lookup(self.0.id)
            .directives
            .iter()
            .map(|id| self.0.ast.read(*id))
    }
}

impl AstId for InterfaceDefinitionId {
    type Reader<'a> = InterfaceDefinition<'a>;
}

impl<'a> From<ReadContext<'a, InterfaceDefinitionId>> for InterfaceDefinition<'a> {
    fn from(value: ReadContext<'a, InterfaceDefinitionId>) -> Self {
        Self(value)
    }
}
