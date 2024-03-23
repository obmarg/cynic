#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    ids::{SchemaDefinitionId, StringLiteralId},
    ReadContext,
    StringLiteral::StringLiteral,
    TypeSystemId,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    AstLookup,
};

pub struct SchemaDefinitionRecord {
    pub description: Option<StringLiteralId>,
    pub roots: Option<RootOperationTypeDefinition>,
}

#[derive(Clone, Copy)]
pub struct SchemaDefinition<'a>(ReadContext<'a, SchemaDefinitionId>);

impl<'a> SchemaDefinition<'a> {
    pub fn description(&self) -> Option<StringLiteral<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .description
            .map(|id| document.read(id))
    }
    pub fn roots(&self) -> impl ExactSizeIterator<Item = RootOperationTypeDefinition> {
        let document = self.0.document;
        document.lookup(self.0.id).roots
    }
}

impl TypeSystemId for SchemaDefinitionId {
    type Reader<'a> = SchemaDefinition<'a>;
}

impl<'a> From<ReadContext<'a, SchemaDefinitionId>> for SchemaDefinition<'a> {
    fn from(value: ReadContext<'a, SchemaDefinitionId>) -> Self {
        Self(value)
    }
}