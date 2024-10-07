use super::prelude::*;
use super::{
    descriptions::Description,
    directives::Directive,
    ids::{DescriptionId, DirectiveId, UnionDefinitionId, UnionMemberId},
    TypeSystemId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct UnionDefinitionRecord {
    pub name: StringId,
    pub description: Option<DescriptionId>,
    pub members: IdRange<UnionMemberId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct UnionDefinition<'a>(pub(in super::super) ReadContext<'a, UnionDefinitionId>);

impl<'a> UnionDefinition<'a> {
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
    pub fn members(&self) -> Iter<'a, UnionMember<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).members, document)
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

impl UnionDefinition<'_> {
    pub fn id(&self) -> UnionDefinitionId {
        self.0.id
    }
}

impl fmt::Debug for UnionDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnionDefinition")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("members", &self.members())
            .field("directives", &self.directives())
            .field("span", &self.span())
            .finish()
    }
}

impl TypeSystemId for UnionDefinitionId {
    type Reader<'a> = UnionDefinition<'a>;
    fn read(self, document: &TypeSystemDocument) -> Self::Reader<'_> {
        UnionDefinition(ReadContext { id: self, document })
    }
}

impl<'a> IdReader<'a> for UnionDefinition<'a> {
    type Id = UnionDefinitionId;
    fn new(id: Self::Id, document: &'a TypeSystemDocument) -> Self {
        document.read(id)
    }
}

pub struct UnionMemberRecord {
    pub name: StringId,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct UnionMember<'a>(pub(in super::super) ReadContext<'a, UnionMemberId>);

impl<'a> UnionMember<'a> {
    pub fn name(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).name)
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl UnionMember<'_> {
    pub fn id(&self) -> UnionMemberId {
        self.0.id
    }
}

impl fmt::Debug for UnionMember<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnionMember")
            .field("name", &self.name())
            .field("span", &self.span())
            .finish()
    }
}

impl TypeSystemId for UnionMemberId {
    type Reader<'a> = UnionMember<'a>;
    fn read(self, document: &TypeSystemDocument) -> Self::Reader<'_> {
        UnionMember(ReadContext { id: self, document })
    }
}

impl<'a> IdReader<'a> for UnionMember<'a> {
    type Id = UnionMemberId;
    fn new(id: Self::Id, document: &'a TypeSystemDocument) -> Self {
        document.read(id)
    }
}