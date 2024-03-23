#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    directives::Directive,
    ids::{DirectiveId, ScalarDefinitionId, StringLiteralId},
    ReadContext,
    StringLiteral::StringLiteral,
    TypeSystemId,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    AstLookup,
};

pub struct ScalarDefinitionRecord {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Option<Span>,
}

#[derive(Clone, Copy)]
pub struct ScalarDefinition<'a>(ReadContext<'a, ScalarDefinitionId>);

impl<'a> ScalarDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let ast = &self.0.document;
        ast.lookup(ast.lookup(self.0.id).name)
    }
    pub fn description(&self) -> Option<StringLiteral<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .description
            .map(|id| document.read(id))
    }
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .directives
            .iter()
            .map(|id| document.read(id))
    }
    pub fn span(&self) -> Option<Span> {
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