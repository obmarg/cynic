#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    arguments::Argument,
    ids::{
        ArgumentId, DirectiveDefinitionId, DirectiveId, InputValueDefinitionId, StringLiteralId,
    },
    input_values::InputValueDefinition,
    strings::StringLiteral,
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    type_system::DirectiveLocation,
    AstLookup, Span,
};

pub struct DirectiveDefinitionRecord {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub arguments: IdRange<InputValueDefinitionId>,
    pub is_repeatable: bool,
    pub locations: Vec<DirectiveLocation>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct DirectiveDefinition<'a>(ReadContext<'a, DirectiveDefinitionId>);

impl<'a> DirectiveDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).name)
    }
    pub fn description(&self) -> Option<StringLiteral<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .description
            .map(|id| document.read(id))
    }
    pub fn arguments(&self) -> impl ExactSizeIterator<Item = InputValueDefinition<'a>> + 'a {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .arguments
            .iter()
            .map(|id| document.read(id))
    }
    pub fn is_repeatable(&self) -> bool {
        let document = self.0.document;
        document.lookup(self.0.id).is_repeatable
    }
    pub fn locations(&self) -> impl ExactSizeIterator<Item = DirectiveLocation> + 'a {
        let document = self.0.document;
        document.lookup(self.0.id).locations.iter().cloned()
    }
    pub fn span(&self) -> Span {
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
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).name)
    }
    pub fn arguments(&self) -> impl ExactSizeIterator<Item = Argument<'a>> + 'a {
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
