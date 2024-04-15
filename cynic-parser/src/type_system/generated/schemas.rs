#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    directives::Directive,
    ids::{DirectiveId, RootOperationTypeDefinitionId, SchemaDefinitionId, StringLiteralId},
    strings::StringLiteral,
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    type_system::DirectiveLocation,
    AstLookup, Span,
};

pub struct SchemaDefinitionRecord {
    pub description: Option<StringLiteralId>,
    pub directives: IdRange<DirectiveId>,
    pub root_operations: IdRange<RootOperationTypeDefinitionId>,
}

#[derive(Clone, Copy)]
pub struct SchemaDefinition<'a>(ReadContext<'a, SchemaDefinitionId>);

impl<'a> SchemaDefinition<'a> {
    pub fn description(&self) -> Option<StringLiteral<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .description
            .map(|id| document.read(id))
    }
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .directives
            .iter()
            .map(|id| document.read(id))
    }
    pub fn root_operations(
        &self,
    ) -> impl ExactSizeIterator<Item = RootOperationTypeDefinition<'a>> + 'a {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .root_operations
            .iter()
            .map(|id| document.read(id))
    }
}

impl TypeSystemId for SchemaDefinitionId {
    type Reader<'a> = SchemaDefinition<'a>;
}

impl<'a> From<ReadContext<'a, SchemaDefinitionId>> for SchemaDefinition<'a> {
    fn from(value: ReadContext<'a, SchemaDefinitionId>) -> Self {
        Self(value)
    }
}

pub struct RootOperationTypeDefinitionRecord {
    pub operation_type: OperationType,
    pub named_type: StringId,
}

#[derive(Clone, Copy)]
pub struct RootOperationTypeDefinition<'a>(ReadContext<'a, RootOperationTypeDefinitionId>);

impl<'a> RootOperationTypeDefinition<'a> {
    pub fn operation_type(&self) -> OperationType {
        let document = self.0.document;
        document.lookup(self.0.id).operation_type
    }
    pub fn named_type(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).named_type)
    }
}

impl TypeSystemId for RootOperationTypeDefinitionId {
    type Reader<'a> = RootOperationTypeDefinition<'a>;
}

impl<'a> From<ReadContext<'a, RootOperationTypeDefinitionId>> for RootOperationTypeDefinition<'a> {
    fn from(value: ReadContext<'a, RootOperationTypeDefinitionId>) -> Self {
        Self(value)
    }
}
