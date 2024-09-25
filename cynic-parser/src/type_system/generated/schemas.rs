use super::prelude::*;
use super::{
    descriptions::Description,
    directives::Directive,
    ids::{DescriptionId, DirectiveId, RootOperationTypeDefinitionId, SchemaDefinitionId},
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct SchemaDefinitionRecord {
    pub description: Option<DescriptionId>,
    pub directives: IdRange<DirectiveId>,
    pub root_operations: IdRange<RootOperationTypeDefinitionId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct SchemaDefinition<'a>(pub(in super::super) ReadContext<'a, SchemaDefinitionId>);

impl<'a> SchemaDefinition<'a> {
    pub fn description(&self) -> Option<Description<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .description
            .map(|id| document.read(id))
    }
    pub fn directives(&self) -> Iter<'a, Directive<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).directives, document)
    }
    pub fn root_operations(&self) -> Iter<'a, RootOperationTypeDefinition<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).root_operations, document)
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl SchemaDefinition<'_> {
    pub fn id(&self) -> SchemaDefinitionId {
        self.0.id
    }
}

impl fmt::Debug for SchemaDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SchemaDefinition")
            .field("description", &self.description())
            .field("directives", &self.directives())
            .field("root_operations", &self.root_operations())
            .field("span", &self.span())
            .finish()
    }
}

impl TypeSystemId for SchemaDefinitionId {
    type Reader<'a> = SchemaDefinition<'a>;
}

impl IdReader for SchemaDefinition<'_> {
    type Id = SchemaDefinitionId;
}

impl<'a> From<ReadContext<'a, SchemaDefinitionId>> for SchemaDefinition<'a> {
    fn from(value: ReadContext<'a, SchemaDefinitionId>) -> Self {
        Self(value)
    }
}

pub struct RootOperationTypeDefinitionRecord {
    pub operation_type: OperationType,
    pub named_type: StringId,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct RootOperationTypeDefinition<'a>(
    pub(in super::super) ReadContext<'a, RootOperationTypeDefinitionId>,
);

impl<'a> RootOperationTypeDefinition<'a> {
    pub fn operation_type(&self) -> OperationType {
        let document = self.0.document;
        document.lookup(self.0.id).operation_type
    }
    pub fn named_type(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).named_type)
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl RootOperationTypeDefinition<'_> {
    pub fn id(&self) -> RootOperationTypeDefinitionId {
        self.0.id
    }
}

impl fmt::Debug for RootOperationTypeDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RootOperationTypeDefinition")
            .field("operation_type", &self.operation_type())
            .field("named_type", &self.named_type())
            .field("span", &self.span())
            .finish()
    }
}

impl TypeSystemId for RootOperationTypeDefinitionId {
    type Reader<'a> = RootOperationTypeDefinition<'a>;
}

impl IdReader for RootOperationTypeDefinition<'_> {
    type Id = RootOperationTypeDefinitionId;
}

impl<'a> From<ReadContext<'a, RootOperationTypeDefinitionId>> for RootOperationTypeDefinition<'a> {
    fn from(value: ReadContext<'a, RootOperationTypeDefinitionId>) -> Self {
        Self(value)
    }
}
