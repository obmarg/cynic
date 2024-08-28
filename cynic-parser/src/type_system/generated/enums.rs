use super::prelude::*;
use super::{
    descriptions::Description,
    directives::Directive,
    ids::{DescriptionId, DirectiveId, EnumDefinitionId, EnumValueDefinitionId},
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct EnumDefinitionRecord {
    pub name: StringId,
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
}

impl IdReader for EnumDefinition<'_> {
    type Id = EnumDefinitionId;
}

impl<'a> From<ReadContext<'a, EnumDefinitionId>> for EnumDefinition<'a> {
    fn from(value: ReadContext<'a, EnumDefinitionId>) -> Self {
        Self(value)
    }
}

pub struct EnumValueDefinitionRecord {
    pub value: StringId,
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
}

impl IdReader for EnumValueDefinition<'_> {
    type Id = EnumValueDefinitionId;
}

impl<'a> From<ReadContext<'a, EnumValueDefinitionId>> for EnumValueDefinition<'a> {
    fn from(value: ReadContext<'a, EnumValueDefinitionId>) -> Self {
        Self(value)
    }
}
