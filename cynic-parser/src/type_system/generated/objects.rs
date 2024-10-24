use super::prelude::*;
use super::{
    descriptions::Description,
    directives::Directive,
    fields::FieldDefinition,
    ids::{DescriptionId, DirectiveId, FieldDefinitionId, ObjectDefinitionId},
    TypeSystemId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct ObjectDefinitionRecord {
    pub name: StringId,
    pub name_span: Span,
    pub description: Option<DescriptionId>,
    pub fields: IdRange<FieldDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub implements_interfaces: Vec<StringId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct ObjectDefinition<'a>(pub(in super::super) ReadContext<'a, ObjectDefinitionId>);

impl<'a> ObjectDefinition<'a> {
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

impl ObjectDefinition<'_> {
    pub fn id(&self) -> ObjectDefinitionId {
        self.0.id
    }
}

impl fmt::Debug for ObjectDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectDefinition")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("fields", &self.fields())
            .field("directives", &self.directives())
            .field(
                "implements_interfaces",
                &self.implements_interfaces().collect::<Vec<_>>(),
            )
            .field("span", &self.span())
            .finish()
    }
}

impl TypeSystemId for ObjectDefinitionId {
    type Reader<'a> = ObjectDefinition<'a>;
    fn read(self, document: &TypeSystemDocument) -> Self::Reader<'_> {
        ObjectDefinition(ReadContext { id: self, document })
    }
}

impl<'a> IdReader<'a> for ObjectDefinition<'a> {
    type Id = ObjectDefinitionId;
    fn new(id: Self::Id, document: &'a TypeSystemDocument) -> Self {
        document.read(id)
    }
}
