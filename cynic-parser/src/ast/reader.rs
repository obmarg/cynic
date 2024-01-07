use crate::Ast;

use super::{
    ids::{AstId, AstLookup, InputValueDefinitionId},
    FieldDefinitionId, InputObjectDefinitionId, NodeContents, ObjectDefinitionId,
    SchemaDefinitionId,
};

pub struct AstReader<'a, I> {
    id: I,
    ast: &'a Ast,
}

impl super::Ast {
    pub fn reader(&self) -> AstReader<'_, ()> {
        AstReader { id: (), ast: self }
    }

    pub fn read<Id>(&self, id: Id) -> AstReader<'_, Id>
    where
        Id: AstId,
    {
        AstReader { id, ast: self }
    }
}

impl<'a> AstReader<'a, ()> {
    pub fn definitions(&self) -> impl Iterator<Item = Definition<'a>> + 'a {
        self.ast.definition_nodes.iter().map(|definition| {
            match self.ast.nodes[definition.0].contents {
                NodeContents::SchemaDefinition(id) => {
                    Definition::Schema(AstReader { id, ast: self.ast })
                }
                NodeContents::ObjectDefiniton(id) => {
                    Definition::Object(AstReader { id, ast: self.ast })
                }
                NodeContents::InputObjectDefiniton(id) => {
                    Definition::InputObject(AstReader { id, ast: self.ast })
                }
                NodeContents::FieldDefinition(_)
                | NodeContents::InputValueDefinition(_)
                | NodeContents::StringLiteral(_) => unreachable!(),
                NodeContents::Ident(_) => unreachable!(),
            }
        })
    }
}

pub enum Definition<'a> {
    Schema(AstReader<'a, SchemaDefinitionId>),
    Object(AstReader<'a, ObjectDefinitionId>),
    InputObject(AstReader<'a, InputObjectDefinitionId>),
}

impl<'a> AstReader<'a, ObjectDefinitionId> {
    pub fn name(&self) -> &str {
        match self.ast.lookup(self.ast.lookup(self.id).name).contents {
            NodeContents::Ident(id) => self.ast.lookup(id),
            _ => unreachable!(),
        }
    }

    pub fn fields(&self) -> impl Iterator<Item = AstReader<'a, FieldDefinitionId>> + 'a {
        self.ast
            .lookup(self.id)
            .fields
            .iter()
            .map(|node| match self.ast.lookup(*node).contents {
                NodeContents::FieldDefinition(id) => self.ast.read(id),
                _ => unreachable!(),
            })
    }
}

impl<'a> AstReader<'a, InputObjectDefinitionId> {
    pub fn name(&self) -> &str {
        match self.ast.lookup(self.ast.lookup(self.id).name).contents {
            NodeContents::Ident(id) => self.ast.lookup(id),
            _ => unreachable!(),
        }
    }

    pub fn fields(&self) -> impl Iterator<Item = AstReader<'a, InputValueDefinitionId>> + 'a {
        self.ast
            .lookup(self.id)
            .fields
            .iter()
            .map(|node| match self.ast.lookup(*node).contents {
                NodeContents::InputValueDefinition(id) => self.ast.read(id),
                _ => unreachable!(),
            })
    }
}

impl<'a> AstReader<'a, FieldDefinitionId> {
    pub fn name(&self) -> &str {
        match self.ast.lookup(self.ast.lookup(self.id).name).contents {
            NodeContents::Ident(id) => self.ast.lookup(id),
            _ => unreachable!(),
        }
    }

    // pub fn ty(&self) -> AstReader<'a, TypeId> {}

    pub fn arguments(&self) -> impl Iterator<Item = AstReader<'a, InputValueDefinitionId>> {
        self.ast.lookup(self.id).arguments.iter().map(|node| {
            match self.ast.lookup(*node).contents {
                NodeContents::InputValueDefinition(id) => self.ast.read(id),
                _ => unreachable!(),
            }
        })
    }
}

impl<'a> AstReader<'a, InputValueDefinitionId> {
    pub fn name(&self) -> &str {
        match self.ast.lookup(self.ast.lookup(self.id).name).contents {
            NodeContents::Ident(id) => self.ast.lookup(id),
            _ => unreachable!(),
        }
    }

    pub fn ty(&self) -> &str {
        match self.ast.lookup(self.ast.lookup(self.id).ty).contents {
            NodeContents::Ident(id) => self.ast.lookup(id),
            _ => unreachable!(),
        }
    }
}
