use super::prelude::*;
use super::{
    directive::Directive,
    ids::{ConstValueId, DirectiveId, NameId, TypeId, VariableDefinitionId},
    name::Name,
    types::Type,
    value::ConstValue,
    ExecutableId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct VariableDefinitionRecord {
    pub name: NameId,
    pub ty: TypeId,
    pub default_value: Option<ConstValueId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct VariableDefinition<'a>(pub(in super::super) ReadContext<'a, VariableDefinitionId>);

impl<'a> VariableDefinition<'a> {
    pub fn name(&self) -> Name<'a> {
        let document = self.0.document;
        document.read(document.lookup(self.0.id).name)
    }
    pub fn ty(&self) -> Type<'a> {
        let document = self.0.document;
        document.read(document.lookup(self.0.id).ty)
    }
    pub fn default_value(&self) -> Option<ConstValue<'a>> {
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
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl VariableDefinition<'_> {
    pub fn id(&self) -> VariableDefinitionId {
        self.0.id
    }
}

impl fmt::Debug for VariableDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VariableDefinition")
            .field("name", &self.name())
            .field("ty", &self.ty())
            .field("default_value", &self.default_value())
            .field("directives", &self.directives())
            .field("span", &self.span())
            .finish()
    }
}

impl ExecutableId for VariableDefinitionId {
    type Reader<'a> = VariableDefinition<'a>;
    fn read(self, document: &ExecutableDocument) -> Self::Reader<'_> {
        VariableDefinition(ReadContext { id: self, document })
    }
}

impl IdReader for VariableDefinition<'_> {
    type Id = VariableDefinitionId;
}
