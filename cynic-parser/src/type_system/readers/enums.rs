use crate::type_system::ids::{EnumDefinitionId, EnumValueDefinitionId};
use crate::AstLookup;

use super::StringLiteral;
use super::{directives::Directive, ReadContext, TypeSystemId};

#[derive(Clone, Copy)]
pub struct EnumDefinition<'a>(ReadContext<'a, EnumDefinitionId>);

impl<'a> EnumDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let ast = &self.0.document;

        ast.lookup(ast.lookup(self.0.id).name)
    }

    pub fn description(&self) -> Option<StringLiteral<'a>> {
        let ast = &self.0.document;
        ast.lookup(self.0.id).description.map(|id| ast.read(id))
    }

    pub fn values(&self) -> impl ExactSizeIterator<Item = EnumValueDefinition<'a>> + 'a {
        let ast = &self.0.document;

        ast.lookup(self.0.id).values.iter().map(|id| ast.read(*id))
    }

    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        let ast = &self.0.document;

        ast.lookup(self.0.id)
            .directives
            .iter()
            .map(|id| ast.read(id))
    }
}

#[derive(Clone, Copy)]
pub struct EnumValueDefinition<'a>(ReadContext<'a, EnumValueDefinitionId>);

impl<'a> EnumValueDefinition<'a> {
    pub fn value(&self) -> &'a str {
        let ast = &self.0.document;

        ast.lookup(ast.lookup(self.0.id).value)
    }

    pub fn description(&self) -> Option<StringLiteral<'a>> {
        let ast = &self.0.document;
        ast.lookup(self.0.id).description.map(|id| ast.read(id))
    }

    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> {
        let ast = &self.0.document;
        ast.lookup(self.0.id)
            .directives
            .iter()
            .map(|id| ast.read(id))
    }
}

impl TypeSystemId for EnumDefinitionId {
    type Reader<'a> = EnumDefinition<'a>;
}

// TODO: consider making this a trait, maybe named Reader/Node/not sure...
impl<'a> From<ReadContext<'a, EnumDefinitionId>> for EnumDefinition<'a> {
    fn from(value: ReadContext<'a, EnumDefinitionId>) -> Self {
        Self(value)
    }
}

impl TypeSystemId for EnumValueDefinitionId {
    type Reader<'a> = EnumValueDefinition<'a>;
}

impl<'a> From<ReadContext<'a, EnumValueDefinitionId>> for EnumValueDefinition<'a> {
    fn from(value: ReadContext<'a, EnumValueDefinitionId>) -> Self {
        Self(value)
    }
}
