#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    arguments::Argument,
    ids::{
        ArgumentId, DirectiveDefinitionId, DirectiveId, InputValueDefinitionId, StringLiteralId,
    },
    input_values::InputValueDefinition,
    ReadContext,
    StringLiteral::StringLiteral,
    TypeSystemId,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    AstLookup,
};

pub struct DirectiveDefinitionRecord {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub arguments: IdRange<InputValueDefinitionId>,
    pub repeatable: Option<Boolean>,
    pub locations: DirectiveLocation,
    pub span: Option<Span>,
}

#[derive(Clone, Copy)]
pub struct DirectiveDefinition<'a>(ReadContext<'a, DirectiveDefinitionId>);

impl<'a> DirectiveDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let ast = &self.0.document;
        ast.lookup(ast.lookup(self.0.id).name)
    }
    pub fn description(&self) -> Option<StringLiteral<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .description
            .map(|id| document.read(id))
    }
    pub fn arguments(&self) -> impl ExactSizeIterator<Item = InputValueDefinition<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .arguments
            .iter()
            .map(|id| document.read(id))
    }
    pub fn repeatable(&self) -> Option<Boolean> {
        let document = self.0.document;
        document.lookup(self.0.id).repeatable
    }
    pub fn locations(&self) -> impl ExactSizeIterator<Item = DirectiveLocation> {
        let document = self.0.document;
        document.lookup(self.0.id).locations
    }
    pub fn span(&self) -> Option<Span> {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl TypeSystemId for DirectiveDefinitionId {
    type Reader<'a> = DirectiveDefinition<'a>;
}

impl<'a> From<ReadContext<'a, DirectiveDefinitionId>> for DirectiveDefinition<'a> {
    fn from(value: ReadContext<'a, DirectiveDefinitionId>) -> Self {
        Self(value)
    }
}

pub struct DirectiveRecord {
    pub name: StringId,
    pub arguments: IdRange<ArgumentId>,
}

#[derive(Clone, Copy)]
pub struct Directive<'a>(ReadContext<'a, DirectiveId>);

impl<'a> Directive<'a> {
    pub fn name(&self) -> &'a str {
        let ast = &self.0.document;
        ast.lookup(ast.lookup(self.0.id).name)
    }
    pub fn arguments(&self) -> impl ExactSizeIterator<Item = Argument<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .arguments
            .iter()
            .map(|id| document.read(id))
    }
}

impl TypeSystemId for DirectiveId {
    type Reader<'a> = Directive<'a>;
}

impl<'a> From<ReadContext<'a, DirectiveId>> for Directive<'a> {
    fn from(value: ReadContext<'a, DirectiveId>) -> Self {
        Self(value)
    }
}