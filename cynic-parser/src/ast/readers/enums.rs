use crate::ast::{
    ids::{EnumDefinitionId, EnumValueDefinitionId},
    AstLookup,
};

use super::{directives::Directive, AstId, ReadContext};

pub struct EnumDefinition<'a>(ReadContext<'a, EnumDefinitionId>);

impl<'a> EnumDefinition<'a> {
    pub fn name(&self) -> &str {
        let ast = &self.0.ast;

        ast.lookup(ast.lookup(self.0.id).name)
    }

    pub fn description(&self) -> Option<&str> {
        let ast = &self.0.ast;
        ast.lookup(self.0.id).description.map(|id| ast.lookup(id))
    }

    pub fn values(&self) -> impl Iterator<Item = EnumValueDefinition<'a>> + 'a {
        let ast = &self.0.ast;

        ast.lookup(self.0.id).values.iter().map(|id| ast.read(*id))
    }

    pub fn directives(&self) -> impl Iterator<Item = Directive<'a>> + 'a {
        let ast = &self.0.ast;

        ast.lookup(self.0.id)
            .directives
            .iter()
            .map(|id| ast.read(*id))
    }
}

pub struct EnumValueDefinition<'a>(ReadContext<'a, EnumValueDefinitionId>);

impl<'a> EnumValueDefinition<'a> {
    pub fn value(&self) -> &str {
        let ast = &self.0.ast;

        ast.lookup(ast.lookup(self.0.id).value)
    }

    pub fn description(&self) -> Option<&'a str> {
        let ast = &self.0.ast;
        ast.lookup(self.0.id).description.map(|id| ast.lookup(id))
    }

    pub fn directives(&self) -> impl Iterator<Item = Directive<'a>> {
        let ast = &self.0.ast;
        ast.lookup(self.0.id)
            .directives
            .iter()
            .map(|id| ast.read(*id))
    }
}

impl AstId for EnumDefinitionId {
    type Reader<'a> = EnumDefinition<'a>;
}

// TODO: consider making this a trait, maybe named Reader/Node/not sure...
impl<'a> From<ReadContext<'a, EnumDefinitionId>> for EnumDefinition<'a> {
    fn from(value: ReadContext<'a, EnumDefinitionId>) -> Self {
        Self(value)
    }
}

impl AstId for EnumValueDefinitionId {
    type Reader<'a> = EnumValueDefinition<'a>;
}

impl<'a> From<ReadContext<'a, EnumValueDefinitionId>> for EnumValueDefinition<'a> {
    fn from(value: ReadContext<'a, EnumValueDefinitionId>) -> Self {
        Self(value)
    }
}
