#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    directives::Directive,
    fields::FieldDefinition,
    ids::{DirectiveId, FieldDefinitionId, ObjectDefinitionId, StringLiteralId},
    strings::StringLiteral,
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    type_system::DirectiveLocation,
    AstLookup, Span,
};

pub struct ObjectDefinitionRecord {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub fields: IdRange<FieldDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub implements_interfaces: Vec<StringId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct ObjectDefinition<'a>(ReadContext<'a, ObjectDefinitionId>);

impl<'a> ObjectDefinition<'a> {
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
    pub fn fields(&self) -> impl ExactSizeIterator<Item = FieldDefinition<'a>> + 'a {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .fields
            .iter()
            .map(|id| document.read(id))
    }
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .directives
            .iter()
            .map(|id| document.read(id))
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

impl TypeSystemId for ObjectDefinitionId {
    type Reader<'a> = ObjectDefinition<'a>;
}

impl<'a> From<ReadContext<'a, ObjectDefinitionId>> for ObjectDefinition<'a> {
    fn from(value: ReadContext<'a, ObjectDefinitionId>) -> Self {
        Self(value)
    }
}