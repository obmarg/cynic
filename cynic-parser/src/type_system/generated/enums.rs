use super::prelude::*;
use super::{
    descriptions::Description,
    directives::Directive,
    ids::{DescriptionId, DirectiveId, EnumDefinitionId, EnumValueDefinitionId},
    TypeSystemId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct EnumDefinitionRecord {
    pub name: StringId,
    pub name_span: Span,
    pub description: Option<DescriptionId>,
    pub values: IdRange<EnumValueDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct EnumDefinition<'a>(pub(in super::super) ReadContext<'a, EnumDefinitionId>);

impl<'a> EnumDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).name)
    }
    pub fn name_span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).name_span
    }
    pub fn description(&self) -> Option<Description<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .description
            .map(|id| document.read(id))
    }
    pub fn values(&self) -> Iter<'a, EnumValueDefinition<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).values, document)
    }
    pub fn directives(&self) -> Iter<'a, Directive<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).directives, document)
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl EnumDefinition<'_> {
    pub fn id(&self) -> EnumDefinitionId {
        self.0.id
    }
}

impl fmt::Debug for EnumDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnumDefinition")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("values", &self.values())
            .field("directives", &self.directives())
            .field("span", &self.span())
            .finish()
    }
}

impl TypeSystemId for EnumDefinitionId {
    type Reader<'a> = EnumDefinition<'a>;
    fn read(self, document: &TypeSystemDocument) -> Self::Reader<'_> {
        EnumDefinition(ReadContext { id: self, document })
    }
}

impl IdReader for EnumDefinition<'_> {
    type Id = EnumDefinitionId;
    type Reader<'a> = EnumDefinition<'a>;
    fn new(id: Self::Id, document: &'_ TypeSystemDocument) -> Self::Reader<'_> {
        document.read(id)
    }
}

pub struct EnumValueDefinitionRecord {
    pub value: StringId,
    pub value_span: Span,
    pub description: Option<DescriptionId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct EnumValueDefinition<'a>(pub(in super::super) ReadContext<'a, EnumValueDefinitionId>);

impl<'a> EnumValueDefinition<'a> {
    pub fn value(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).value)
    }
    pub fn value_span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).value_span
    }
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
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl EnumValueDefinition<'_> {
    pub fn id(&self) -> EnumValueDefinitionId {
        self.0.id
    }
}

impl fmt::Debug for EnumValueDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnumValueDefinition")
            .field("value", &self.value())
            .field("description", &self.description())
            .field("directives", &self.directives())
            .field("span", &self.span())
            .finish()
    }
}

impl TypeSystemId for EnumValueDefinitionId {
    type Reader<'a> = EnumValueDefinition<'a>;
    fn read(self, document: &TypeSystemDocument) -> Self::Reader<'_> {
        EnumValueDefinition(ReadContext { id: self, document })
    }
}

impl IdReader for EnumValueDefinition<'_> {
    type Id = EnumValueDefinitionId;
    type Reader<'a> = EnumValueDefinition<'a>;
    fn new(id: Self::Id, document: &'_ TypeSystemDocument) -> Self::Reader<'_> {
        document.read(id)
    }
}
