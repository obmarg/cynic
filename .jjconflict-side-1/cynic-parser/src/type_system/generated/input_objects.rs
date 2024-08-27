use super::prelude::*;
use super::{
    descriptions::Description,
    directives::Directive,
    ids::{DescriptionId, DirectiveId, InputObjectDefinitionId, InputValueDefinitionId},
    input_values::InputValueDefinition,
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct InputObjectDefinitionRecord {
    pub name: StringId,
    pub description: Option<DescriptionId>,
    pub fields: IdRange<InputValueDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct InputObjectDefinition<'a>(pub(in super::super) ReadContext<'a, InputObjectDefinitionId>);

impl<'a> InputObjectDefinition<'a> {
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
    pub fn fields(&self) -> Iter<'a, InputValueDefinition<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).fields, document)
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

impl InputObjectDefinition<'_> {
    pub fn id(&self) -> InputObjectDefinitionId {
        self.0.id
    }
}

impl fmt::Debug for InputObjectDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InputObjectDefinition")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("fields", &self.fields())
            .field("directives", &self.directives())
            .field("span", &self.span())
            .finish()
    }
}

impl TypeSystemId for InputObjectDefinitionId {
    type Reader<'a> = InputObjectDefinition<'a>;
}

impl IdReader for InputObjectDefinition<'_> {
    type Id = InputObjectDefinitionId;
}

impl<'a> From<ReadContext<'a, InputObjectDefinitionId>> for InputObjectDefinition<'a> {
    fn from(value: ReadContext<'a, InputObjectDefinitionId>) -> Self {
        Self(value)
    }
}
