use super::prelude::*;
use super::{
    arguments::Argument,
    descriptions::Description,
    ids::{ArgumentId, DescriptionId, DirectiveDefinitionId, DirectiveId, InputValueDefinitionId},
    input_values::InputValueDefinition,
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct DirectiveDefinitionRecord {
    pub name: StringId,
    pub description: Option<DescriptionId>,
    pub arguments: IdRange<InputValueDefinitionId>,
    pub is_repeatable: bool,
    pub locations: Vec<DirectiveLocation>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct DirectiveDefinition<'a>(pub(in super::super) ReadContext<'a, DirectiveDefinitionId>);

impl<'a> DirectiveDefinition<'a> {
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
    pub fn arguments(&self) -> Iter<'a, InputValueDefinition<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).arguments, document)
    }
    pub fn is_repeatable(&self) -> bool {
        let document = self.0.document;
        document.lookup(self.0.id).is_repeatable
    }
    pub fn locations(&self) -> impl ExactSizeIterator<Item = DirectiveLocation> + 'a {
        let document = self.0.document;
        document.lookup(self.0.id).locations.iter().copied()
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl DirectiveDefinition<'_> {
    pub fn id(&self) -> DirectiveDefinitionId {
        self.0.id
    }
}

impl fmt::Debug for DirectiveDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DirectiveDefinition")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("arguments", &self.arguments())
            .field("is_repeatable", &self.is_repeatable())
            .field("locations", &self.locations().collect::<Vec<_>>())
            .field("span", &self.span())
            .finish()
    }
}

impl TypeSystemId for DirectiveDefinitionId {
    type Reader<'a> = DirectiveDefinition<'a>;
}

impl IdReader for DirectiveDefinition<'_> {
    type Id = DirectiveDefinitionId;
}

impl<'a> From<ReadContext<'a, DirectiveDefinitionId>> for DirectiveDefinition<'a> {
    fn from(value: ReadContext<'a, DirectiveDefinitionId>) -> Self {
        Self(value)
    }
}

pub struct DirectiveRecord {
    pub name: StringId,
    pub arguments: IdRange<ArgumentId>,
}

#[derive(Clone, Copy)]
pub struct Directive<'a>(pub(in super::super) ReadContext<'a, DirectiveId>);

impl<'a> Directive<'a> {
    pub fn name(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).name)
    }
    pub fn arguments(&self) -> Iter<'a, Argument<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).arguments, document)
    }
}

impl Directive<'_> {
    pub fn id(&self) -> DirectiveId {
        self.0.id
    }
}

impl fmt::Debug for Directive<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Directive")
            .field("name", &self.name())
            .field("arguments", &self.arguments())
            .finish()
    }
}

impl TypeSystemId for DirectiveId {
    type Reader<'a> = Directive<'a>;
}

impl IdReader for Directive<'_> {
    type Id = DirectiveId;
}

impl<'a> From<ReadContext<'a, DirectiveId>> for Directive<'a> {
    fn from(value: ReadContext<'a, DirectiveId>) -> Self {
        Self(value)
    }
}
