#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    directives::Directive,
    ids::{DirectiveId, ScalarDefinitionId, StringLiteralId},
    strings::StringLiteral,
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    type_system::DirectiveLocation,
    AstLookup, Span,
};

pub struct ScalarDefinitionRecord {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct ScalarDefinition<'a>(ReadContext<'a, ScalarDefinitionId>);

impl<'a> ScalarDefinition<'a> {
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
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .directives
            .iter()
            .map(|id| document.read(id))
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl TypeSystemId for ScalarDefinitionId {
    type Reader<'a> = ScalarDefinition<'a>;
}

impl<'a> From<ReadContext<'a, ScalarDefinitionId>> for ScalarDefinition<'a> {
    fn from(value: ReadContext<'a, ScalarDefinitionId>) -> Self {
        Self(value)
    }
}
