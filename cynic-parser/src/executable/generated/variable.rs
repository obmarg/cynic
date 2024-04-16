use super::{
    directive::Directive,
    ids::{DirectiveId, TypeId, ValueId, VariableDefinitionId},
    types::Type,
    value::Value,
    ExecutableId, ReadContext,
};
#[allow(unused_imports)]
use super::{ids::StringId, Iter};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    AstLookup,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct VariableDefinitionRecord {
    pub name: StringId,
    pub ty: TypeId,
    pub default_value: Option<ValueId>,
    pub directives: IdRange<DirectiveId>,
}

#[derive(Clone, Copy)]
pub struct VariableDefinition<'a>(ReadContext<'a, VariableDefinitionId>);

impl<'a> VariableDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).name)
    }
    pub fn ty(&self) -> Type<'a> {
        let document = self.0.document;
        document.read(document.lookup(self.0.id).ty)
    }
    pub fn default_value(&self) -> Option<Value<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .default_value
            .map(|id| document.read(id))
    }
    pub fn directives(&self) -> Iter<'a, Directive<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).directives, document)
    }
}

impl fmt::Debug for VariableDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VariableDefinition")
            .field("name", &self.name())
            .field("ty", &self.ty())
            .field("default_value", &self.default_value())
            .field("directives", &self.directives().collect::<Vec<_>>())
            .finish()
    }
}

impl ExecutableId for VariableDefinitionId {
    type Reader<'a> = VariableDefinition<'a>;
}

impl super::IdReader for VariableDefinition<'_> {
    type Id = VariableDefinitionId;
}

impl<'a> From<ReadContext<'a, VariableDefinitionId>> for VariableDefinition<'a> {
    fn from(value: ReadContext<'a, VariableDefinitionId>) -> Self {
        Self(value)
    }
}