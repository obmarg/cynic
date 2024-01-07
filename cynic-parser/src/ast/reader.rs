use crate::Ast;

use super::{
    ids::{ArgumentId, AstId, AstLookup, DirectiveId, InputValueDefinitionId, TypeId, ValueId},
    FieldDefinitionId, InputObjectDefinitionId, NodeContents, ObjectDefinitionId, OperationType,
    SchemaDefinitionId, Type, WrappingType,
};

pub struct AstReader<'a, I> {
    id: I,
    ast: &'a Ast,
}

impl super::Ast {
    pub fn read<Id>(&self, id: Id) -> AstReader<'_, Id>
    where
        Id: AstId,
    {
        AstReader { id, ast: self }
    }
}

impl Ast {
    pub fn definitions<'a>(&'a self) -> impl Iterator<Item = Definition<'a>> + 'a {
        self.definition_nodes
            .iter()
            .map(|definition| match self.nodes[definition.0].contents {
                NodeContents::SchemaDefinition(id) => Definition::Schema(self.read(id)),
                NodeContents::ObjectDefiniton(id) => Definition::Object(self.read(id)),
                NodeContents::InputObjectDefiniton(id) => Definition::InputObject(self.read(id)),
                NodeContents::FieldDefinition(_)
                | NodeContents::InputValueDefinition(_)
                | NodeContents::StringLiteral(_) => unreachable!(),
                NodeContents::Ident(_) => unreachable!(),
            })
    }
}

pub enum Definition<'a> {
    Schema(AstReader<'a, SchemaDefinitionId>),
    Object(AstReader<'a, ObjectDefinitionId>),
    InputObject(AstReader<'a, InputObjectDefinitionId>),
}

impl<'a> AstReader<'a, SchemaDefinitionId> {
    pub fn root_operations(&self) -> impl Iterator<Item = (OperationType, &'a str)> {
        self.ast
            .lookup(self.id)
            .roots
            .iter()
            .map(|root| (root.operation_type, self.ast.lookup(root.named_type)))
    }
}

impl<'a> AstReader<'a, ObjectDefinitionId> {
    pub fn name(&self) -> &str {
        self.ast.lookup(self.ast.lookup(self.id).name)
    }

    pub fn implements_interfaces(&self) -> impl Iterator<Item = &'a str> + 'a {
        self.ast
            .lookup(self.id)
            .implements
            .iter()
            .map(|id| self.ast.lookup(*id))
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

    pub fn directives(&self) -> impl Iterator<Item = AstReader<'a, DirectiveId>> + 'a {
        self.ast
            .lookup(self.id)
            .directives
            .iter()
            .map(|id| self.ast.read(*id))
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

    pub fn directives(&self) -> impl Iterator<Item = AstReader<'a, DirectiveId>> + 'a {
        self.ast
            .lookup(self.id)
            .directives
            .iter()
            .map(|id| self.ast.read(*id))
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

    pub fn directives(&self) -> impl Iterator<Item = AstReader<'a, DirectiveId>> + 'a {
        self.ast
            .lookup(self.id)
            .directives
            .iter()
            .map(|id| self.ast.read(*id))
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

    pub fn directives(&self) -> impl Iterator<Item = AstReader<'a, DirectiveId>> + 'a {
        self.ast
            .lookup(self.id)
            .directives
            .iter()
            .map(|id| self.ast.read(*id))
    }
}

impl<'a> AstReader<'a, DirectiveId> {
    pub fn name(&self) -> &str {
        self.ast.lookup(self.ast.lookup(self.id).name)
    }

    pub fn arguments(&self) -> impl Iterator<Item = AstReader<'a, ArgumentId>> {
        self.ast
            .lookup(self.id)
            .arguments
            .iter()
            .map(|id| self.ast.read(*id))
    }
}

impl<'a> AstReader<'a, ArgumentId> {
    pub fn name(&self) -> &str {
        self.ast.lookup(self.ast.lookup(self.id).name)
    }

    pub fn value(&self) -> AstReader<'a, ValueId> {
        self.ast.read(self.ast.lookup(self.id).value)
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
