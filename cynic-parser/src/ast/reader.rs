use crate::Ast;

use super::{
    ids::InputValueDefinitionId, FieldDefinitionId, InputObjectDefinitionId, NodeContents, NodeId,
    ObjectDefinitionId, SchemaDefinitionId,
};

pub struct AstReader<'a, I> {
    id: I,
    ast: &'a Ast,
}

impl super::Ast {
    pub fn reader(&self) -> AstReader<'_, ()> {
        AstReader { id: (), ast: self }
    }

    // fn lookup(&self, )
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
        match self.ast[self.ast[self.id].name].contents {
            NodeContents::Ident(id) => &self.ast[id],
            _ => unreachable!(),
        }
    }

    pub fn fields(&self) -> impl IntoIterator<Item = AstReader<'a, FieldDefinitionId>> + 'a {
        self.ast[self.id]
            .fields
            .iter()
            .map(|node| match self.ast[*node].contents {
                NodeContents::FieldDefinition(id) => AstReader { id, ast: self.ast },
                _ => unreachable!(),
            })
    }
}

impl<'a> AstReader<'a, FieldDefinitionId> {
    pub fn name(&self) -> &str {
        match self.ast[self.ast[self.id].name].contents {
            NodeContents::Ident(id) => &self.ast[id],
            _ => unreachable!(),
        }
    }

    // pub fn ty(&self) -> AstReader<'a, TypeId> {}

    pub fn arguments(&self) -> impl IntoIterator<Item = AstReader<'a, InputValueDefinitionId>> {
        self.ast[self.id]
            .arguments
            .iter()
            .map(|node| match self.ast[*node].contents {
                NodeContents::InputValueDefinition(id) => AstReader { id, ast: self.ast },
                _ => unreachable!(),
            })
    }
}

impl<'a> AstReader<'a, InputValueDefinitionId> {
    pub fn name(&self) -> &str {
        match self.ast[self.ast[self.id].name].contents {
            NodeContents::Ident(id) => &self.ast[id],
            _ => unreachable!(),
        }
    }

    pub fn ty(&self) -> &str {
        match self.ast[self.ast[self.id].ty].contents {
            NodeContents::Ident(id) => &self.ast[id],
            _ => unreachable!(),
        }
    }
}
