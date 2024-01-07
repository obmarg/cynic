use crate::Ast;

use super::{
    ids::{AstId, AstLookup, InputValueDefinitionId, TypeId, ValueId},
    FieldDefinitionId, InputObjectDefinitionId, NodeContents, ObjectDefinitionId,
    SchemaDefinitionId, Type, WrappingType,
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
        self.ast.lookup(self.ast.lookup(self.id).name)
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
        self.ast.lookup(self.ast.lookup(self.id).name)
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
        self.ast.lookup(self.ast.lookup(self.id).name)
    }

    pub fn ty(&self) -> AstReader<'a, TypeId> {
        self.ast.read(self.ast.lookup(self.id).ty)
    }

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
        self.ast.lookup(self.ast.lookup(self.id).name)
    }

    pub fn ty(&self) -> AstReader<'a, TypeId> {
        self.ast.read(self.ast.lookup(self.id).ty)
    }

    pub fn default_value(&self) -> Option<AstReader<'a, ValueId>> {
        self.ast.lookup(self.id).default.map(|id| self.ast.read(id))
    }
}

impl<'a> AstReader<'a, ValueId> {
    pub fn value(&self) -> ValueReader<'a> {
        match self.ast.lookup(self.id) {
            super::Value::Variable(id) => ValueReader::Variable(self.ast.lookup(*id)),
            super::Value::Int(num) => ValueReader::Int(*num),
            super::Value::Float(num) => ValueReader::Float(*num),
            super::Value::String(id) => ValueReader::String(self.ast.lookup(*id)),
            super::Value::Boolean(val) => ValueReader::Boolean(*val),
            super::Value::Null => ValueReader::Null,
            super::Value::Enum(id) => ValueReader::Enum(self.ast.lookup(*id)),
            super::Value::List(ids) => {
                ValueReader::List(ids.iter().map(|id| self.ast.read(*id)).collect())
            }
            super::Value::Object(pairs) => ValueReader::Object(
                pairs
                    .iter()
                    .map(|(name, value)| (self.ast.lookup(*name), self.ast.read(*value)))
                    .collect(),
            ),
        }
    }
}

pub enum ValueReader<'a> {
    Variable(&'a str),
    Int(i32),
    Float(f32),
    String(&'a str),
    Boolean(bool),
    Null,
    Enum(&'a str),
    List(Vec<AstReader<'a, ValueId>>),
    Object(Vec<(&'a str, AstReader<'a, ValueId>)>),
}

impl<'a> AstReader<'a, TypeId> {
    pub fn to_string(&self) -> String {
        let Type { name, wrappers } = self.ast.lookup(self.id);
        let mut output = String::new();
        for wrapping in wrappers.iter().rev() {
            if let WrappingType::List = wrapping {
                output.push('[');
            }
        }
        output.push_str(self.ast.lookup(*name));
        for wrapping in wrappers.iter() {
            match wrapping {
                WrappingType::NonNull => output.push('!'),
                WrappingType::List => output.push(']'),
            }
        }
        output
    }
}
