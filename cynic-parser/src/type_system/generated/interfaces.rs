use super::prelude::*;
use super::{
    directives::Directive,
    fields::FieldDefinition,
    ids::{DirectiveId, FieldDefinitionId, InterfaceDefinitionId, StringLiteralId},
    strings::StringLiteral,
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct InterfaceDefinitionRecord {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub fields: IdRange<FieldDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub implements_interfaces: Vec<StringId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct InterfaceDefinition<'a>(ReadContext<'a, InterfaceDefinitionId>);

impl<'a> InterfaceDefinition<'a> {
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
    pub fn fields(&self) -> Iter<'a, FieldDefinition<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).fields, document)
    }
    pub fn directives(&self) -> Iter<'a, Directive<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).directives, document)
    }
    pub fn implements_interfaces(&self) -> impl ExactSizeIterator<Item = &'a str> + 'a {
        let document = &self.0.document;
        document
            .lookup(self.0.id)
            .implements_interfaces
            .iter()
            .map(|id| document.lookup(*id))
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl fmt::Debug for InterfaceDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InterfaceDefinition")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("fields", &self.fields().collect::<Vec<_>>())
            .field("directives", &self.directives().collect::<Vec<_>>())
            .field(
                "implements_interfaces",
                &self.implements_interfaces().collect::<Vec<_>>(),
            )
            .field("span", &self.span())
            .finish()
    }
}

impl TypeSystemId for InterfaceDefinitionId {
    type Reader<'a> = InterfaceDefinition<'a>;
}

impl IdReader for InterfaceDefinition<'_> {
    type Id = InterfaceDefinitionId;
}

impl<'a> From<ReadContext<'a, InterfaceDefinitionId>> for InterfaceDefinition<'a> {
    fn from(value: ReadContext<'a, InterfaceDefinitionId>) -> Self {
        Self(value)
    }
}