use super::prelude::*;
use super::{
    descriptions::Description,
    directives::Directive,
    ids::{DescriptionId, DirectiveId, FieldDefinitionId, InputValueDefinitionId, TypeId},
    input_values::InputValueDefinition,
    types::Type,
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct FieldDefinitionRecord {
    pub name: StringId,
    pub ty: TypeId,
    pub arguments: IdRange<InputValueDefinitionId>,
    pub description: Option<DescriptionId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct FieldDefinition<'a>(pub(in super::super) ReadContext<'a, FieldDefinitionId>);

impl<'a> FieldDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).name)
    }
    pub fn ty(&self) -> Type<'a> {
        let document = self.0.document;
        document.read(document.lookup(self.0.id).ty)
    }
    pub fn arguments(&self) -> Iter<'a, InputValueDefinition<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).arguments, document)
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

impl FieldDefinition<'_> {
    pub fn id(&self) -> FieldDefinitionId {
        self.0.id
    }
}

impl fmt::Debug for FieldDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FieldDefinition")
            .field("name", &self.name())
            .field("ty", &self.ty())
            .field("arguments", &self.arguments())
            .field("description", &self.description())
            .field("directives", &self.directives())
            .field("span", &self.span())
            .finish()
    }
}

impl TypeSystemId for FieldDefinitionId {
    type Reader<'a> = FieldDefinition<'a>;
}

impl IdReader for FieldDefinition<'_> {
    type Id = FieldDefinitionId;
}

impl<'a> From<ReadContext<'a, FieldDefinitionId>> for FieldDefinition<'a> {
    fn from(value: ReadContext<'a, FieldDefinitionId>) -> Self {
        Self(value)
    }
}
