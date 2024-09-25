use super::prelude::*;
use super::{
    ids::{NameId, VariableValueId},
    names::Name,
    ReadContext, ValueId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct VariableValueRecord {
    pub name: NameId,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct VariableValue<'a>(pub(in super::super) ReadContext<'a, VariableValueId>);

impl<'a> VariableValue<'a> {
    pub fn name(&self) -> Name<'a> {
        let document = self.0.document;
        document.read(document.lookup(self.0.id).name)
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl VariableValue<'_> {
    pub fn id(&self) -> VariableValueId {
        self.0.id
    }
}

impl fmt::Debug for VariableValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VariableValue")
            .field("name", &self.name())
            .field("span", &self.span())
            .finish()
    }
}

impl ValueId for VariableValueId {
    type Reader<'a> = VariableValue<'a>;
}

impl IdReader for VariableValue<'_> {
    type Id = VariableValueId;
}

impl<'a> From<ReadContext<'a, VariableValueId>> for VariableValue<'a> {
    fn from(value: ReadContext<'a, VariableValueId>) -> Self {
        Self(value)
    }
}