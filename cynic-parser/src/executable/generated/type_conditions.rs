use super::prelude::*;
use super::{ids::TypeConditionId, ExecutableId};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct TypeConditionRecord {
    pub on: StringId,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct TypeCondition<'a>(pub(in super::super) ReadContext<'a, TypeConditionId>);

impl<'a> TypeCondition<'a> {
    pub fn on(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).on)
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl TypeCondition<'_> {
    pub fn id(&self) -> TypeConditionId {
        self.0.id
    }
}

impl fmt::Debug for TypeCondition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeCondition")
            .field("on", &self.on())
            .field("span", &self.span())
            .finish()
    }
}

impl ExecutableId for TypeConditionId {
    type Reader<'a> = TypeCondition<'a>;
    fn read(self, document: &ExecutableDocument) -> Self::Reader<'_> {
        TypeCondition(ReadContext { id: self, document })
    }
}

impl IdReader for TypeCondition<'_> {
    type Id = TypeConditionId;
}