use super::prelude::*;
use super::{
    descriptions::Description,
    directives::Directive,
    ids::{DescriptionId, DirectiveId, InputValueDefinitionId, TypeId, ValueId},
    types::Type,
    value::Value,
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct InputValueDefinitionRecord {
    pub name: StringId,
    pub ty: TypeId,
    pub description: Option<DescriptionId>,
    pub default_value: Option<ValueId>,
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
    pub fn default_value(&self) -> Option<Value<'a>> {
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
}

impl IdReader for InputValueDefinition<'_> {
    type Id = InputValueDefinitionId;
}

impl<'a> From<ReadContext<'a, InputValueDefinitionId>> for InputValueDefinition<'a> {
    fn from(value: ReadContext<'a, InputValueDefinitionId>) -> Self {
        Self(value)
    }
}
