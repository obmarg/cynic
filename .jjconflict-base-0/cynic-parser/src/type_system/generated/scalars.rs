use super::prelude::*;
use super::{
    descriptions::Description,
    directives::Directive,
    ids::{DescriptionId, DirectiveId, ScalarDefinitionId},
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct ScalarDefinitionRecord {
    pub name: StringId,
    pub description: Option<DescriptionId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct ScalarDefinition<'a>(pub(in super::super) ReadContext<'a, ScalarDefinitionId>);

impl<'a> ScalarDefinition<'a> {
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
    pub fn directives(&self) -> Iter<'a, Directive<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).directives, document)
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl ScalarDefinition<'_> {
    pub fn id(&self) -> ScalarDefinitionId {
        self.0.id
    }
}

impl fmt::Debug for ScalarDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScalarDefinition")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("directives", &self.directives())
            .field("span", &self.span())
            .finish()
    }
}

impl TypeSystemId for ScalarDefinitionId {
    type Reader<'a> = ScalarDefinition<'a>;
}

impl IdReader for ScalarDefinition<'_> {
    type Id = ScalarDefinitionId;
}

impl<'a> From<ReadContext<'a, ScalarDefinitionId>> for ScalarDefinition<'a> {
    fn from(value: ReadContext<'a, ScalarDefinitionId>) -> Self {
        Self(value)
    }
}
