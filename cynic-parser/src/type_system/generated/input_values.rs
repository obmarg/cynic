use super::prelude::*;
use super::{
    descriptions::Description,
    directives::Directive,
    ids::{ConstValueId, DescriptionId, DirectiveId, InputValueDefinitionId, TypeId},
    types::Type,
    value::ConstValue,
    TypeSystemId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct InputValueDefinitionRecord {
    pub name: StringId,
    pub name_span: Span,
    pub ty: TypeId,
    pub description: Option<DescriptionId>,
    pub default_value: Option<ConstValueId>,
    pub default_value_span: Span,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct InputValueDefinition<'a>(pub(in super::super) ReadContext<'a, InputValueDefinitionId>);

impl<'a> InputValueDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).name)
    }
    pub fn name_span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).name_span
    }
    pub fn ty(&self) -> Type<'a> {
        let document = self.0.document;
        document.read(document.lookup(self.0.id).ty)
    }
    pub fn description(&self) -> Option<Description<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .description
            .map(|id| document.read(id))
    }
    pub fn default_value(&self) -> Option<ConstValue<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .default_value
            .map(|id| document.read(id))
    }
    pub fn default_value_span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).default_value_span
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

impl InputValueDefinition<'_> {
    pub fn id(&self) -> InputValueDefinitionId {
        self.0.id
    }
}

impl fmt::Debug for InputValueDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InputValueDefinition")
            .field("name", &self.name())
            .field("ty", &self.ty())
            .field("description", &self.description())
            .field("default_value", &self.default_value())
            .field("default_value_span", &self.default_value_span())
            .field("directives", &self.directives())
            .field("span", &self.span())
            .finish()
    }
}

impl TypeSystemId for InputValueDefinitionId {
    type Reader<'a> = InputValueDefinition<'a>;
    fn read(self, document: &TypeSystemDocument) -> Self::Reader<'_> {
        InputValueDefinition(ReadContext { id: self, document })
    }
}

impl IdReader for InputValueDefinition<'_> {
    type Id = InputValueDefinitionId;
    type Reader<'a> = InputValueDefinition<'a>;
    fn new(id: Self::Id, document: &'_ TypeSystemDocument) -> Self::Reader<'_> {
        document.read(id)
    }
}
