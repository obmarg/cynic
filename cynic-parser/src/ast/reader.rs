use crate::{ast, Ast};

use super::{
    ids::{
        ArgumentId, AstId, AstLookup, DirectiveDefinitionId, DirectiveId, EnumDefinitionId,
        EnumValueDefinitionId, InputValueDefinitionId, InterfaceDefinitionId, ScalarDefinitionId,
        TypeId, UnionDefinitionId, ValueId,
    },
    DirectiveLocation, FieldDefinitionId, InputObjectDefinitionId, ObjectDefinitionId,
    OperationType, SchemaDefinitionId, Type, WrappingType,
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
    pub fn definitions(&self) -> impl Iterator<Item = Definition<'_>> + '_ {
        self.definitions.iter().map(|definition| match definition {
            ast::AstDefinition::Schema(id) => Definition::Schema(self.read(*id)),
            ast::AstDefinition::Scalar(id) => {
                Definition::Type(TypeDefinition::Scalar(self.read(*id)))
            }
            ast::AstDefinition::Object(id) => {
                Definition::Type(TypeDefinition::Object(self.read(*id)))
            }
            ast::AstDefinition::Interface(id) => {
                Definition::Type(TypeDefinition::Interface(self.read(*id)))
            }
            ast::AstDefinition::Union(id) => {
                Definition::Type(TypeDefinition::Union(self.read(*id)))
            }
            ast::AstDefinition::Enum(id) => Definition::Type(TypeDefinition::Enum(self.read(*id))),
            ast::AstDefinition::InputObject(id) => {
                Definition::Type(TypeDefinition::InputObject(self.read(*id)))
            }
            ast::AstDefinition::SchemaExtension(id) => Definition::SchemaExtension(self.read(*id)),
            ast::AstDefinition::ScalarExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Scalar(self.read(*id)))
            }
            ast::AstDefinition::ObjectExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Object(self.read(*id)))
            }
            ast::AstDefinition::InterfaceExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Interface(self.read(*id)))
            }
            ast::AstDefinition::UnionExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Union(self.read(*id)))
            }
            ast::AstDefinition::EnumExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Enum(self.read(*id)))
            }
            ast::AstDefinition::InputObjectExtension(id) => {
                Definition::TypeExtension(TypeDefinition::InputObject(self.read(*id)))
            }
            ast::AstDefinition::Directive(id) => Definition::Directive(self.read(*id)),
        })
    }
}

pub enum Definition<'a> {
    Schema(AstReader<'a, SchemaDefinitionId>),
    SchemaExtension(AstReader<'a, SchemaDefinitionId>),
    Type(TypeDefinition<'a>),
    TypeExtension(TypeDefinition<'a>),
    Directive(AstReader<'a, DirectiveDefinitionId>),
}

pub enum TypeDefinition<'a> {
    Scalar(AstReader<'a, ScalarDefinitionId>),
    Object(AstReader<'a, ObjectDefinitionId>),
    Interface(AstReader<'a, InterfaceDefinitionId>),
    Union(AstReader<'a, UnionDefinitionId>),
    Enum(AstReader<'a, EnumDefinitionId>),
    InputObject(AstReader<'a, InputObjectDefinitionId>),
}

impl<'a> AstReader<'a, SchemaDefinitionId> {
    pub fn description(&self) -> Option<&str> {
        self.ast
            .lookup(self.id)
            .description
            .map(|id| self.ast.lookup(id))
    }

    pub fn root_operations(&self) -> impl Iterator<Item = (OperationType, &'a str)> {
        self.ast
            .lookup(self.id)
            .roots
            .iter()
            .map(|root| (root.operation_type, self.ast.lookup(root.named_type)))
    }
}

impl<'a> AstReader<'a, ScalarDefinitionId> {
    pub fn name(&self) -> &str {
        self.ast.lookup(self.ast.lookup(self.id).name)
    }
    pub fn description(&self) -> Option<&str> {
        self.ast
            .lookup(self.id)
            .description
            .map(|id| self.ast.lookup(id))
    }

    pub fn directives(&self) -> impl Iterator<Item = AstReader<'a, DirectiveId>> + 'a {
        self.ast
            .lookup(self.id)
            .directives
            .iter()
            .map(|id| self.ast.read(*id))
    }
}

impl<'a> AstReader<'a, ObjectDefinitionId> {
    pub fn name(&self) -> &str {
        self.ast.lookup(self.ast.lookup(self.id).name)
    }
    pub fn description(&self) -> Option<&str> {
        self.ast
            .lookup(self.id)
            .description
            .map(|id| self.ast.lookup(id))
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
            .map(|id| self.ast.read(*id))
    }

    pub fn directives(&self) -> impl Iterator<Item = AstReader<'a, DirectiveId>> + 'a {
        self.ast
            .lookup(self.id)
            .directives
            .iter()
            .map(|id| self.ast.read(*id))
    }
}

impl<'a> AstReader<'a, InterfaceDefinitionId> {
    pub fn name(&self) -> &str {
        self.ast.lookup(self.ast.lookup(self.id).name)
    }

    pub fn description(&self) -> Option<&str> {
        self.ast
            .lookup(self.id)
            .description
            .map(|id| self.ast.lookup(id))
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
            .map(|id| self.ast.read(*id))
    }

    pub fn directives(&self) -> impl Iterator<Item = AstReader<'a, DirectiveId>> + 'a {
        self.ast
            .lookup(self.id)
            .directives
            .iter()
            .map(|id| self.ast.read(*id))
    }
}

impl<'a> AstReader<'a, UnionDefinitionId> {
    pub fn name(&self) -> &str {
        self.ast.lookup(self.ast.lookup(self.id).name)
    }

    pub fn description(&self) -> Option<&str> {
        self.ast
            .lookup(self.id)
            .description
            .map(|id| self.ast.lookup(id))
    }

    pub fn members(&self) -> impl Iterator<Item = &'a str> + 'a {
        self.ast
            .lookup(self.id)
            .members
            .iter()
            .map(|id| self.ast.lookup(*id))
    }

    pub fn directives(&self) -> impl Iterator<Item = AstReader<'a, DirectiveId>> + 'a {
        self.ast
            .lookup(self.id)
            .directives
            .iter()
            .map(|id| self.ast.read(*id))
    }
}

impl<'a> AstReader<'a, EnumDefinitionId> {
    pub fn name(&self) -> &str {
        self.ast.lookup(self.ast.lookup(self.id).name)
    }
    pub fn description(&self) -> Option<&str> {
        self.ast
            .lookup(self.id)
            .description
            .map(|id| self.ast.lookup(id))
    }

    pub fn values(&self) -> impl Iterator<Item = AstReader<'a, EnumValueDefinitionId>> + 'a {
        self.ast
            .lookup(self.id)
            .values
            .iter()
            .map(|id| self.ast.read(*id))
    }

    pub fn directives(&self) -> impl Iterator<Item = AstReader<'a, DirectiveId>> + 'a {
        self.ast
            .lookup(self.id)
            .directives
            .iter()
            .map(|id| self.ast.read(*id))
    }
}

impl<'a> AstReader<'a, EnumValueDefinitionId> {
    pub fn value(&self) -> &str {
        self.ast.lookup(self.ast.lookup(self.id).value)
    }

    pub fn description(&self) -> Option<&'a str> {
        self.ast
            .lookup(self.id)
            .description
            .map(|id| self.ast.lookup(id))
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

    pub fn description(&self) -> Option<&str> {
        self.ast
            .lookup(self.id)
            .description
            .map(|id| self.ast.lookup(id))
    }

    pub fn fields(&self) -> impl Iterator<Item = AstReader<'a, InputValueDefinitionId>> + 'a {
        self.ast
            .lookup(self.id)
            .fields
            .iter()
            .map(|id| self.ast.read(*id))
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
        self.ast
            .lookup(self.id)
            .arguments
            .iter()
            .map(|id| self.ast.read(*id))
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

    pub fn description(&self) -> Option<&'a str> {
        self.ast
            .lookup(self.id)
            .description
            .map(|id| self.ast.lookup(id))
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

impl<'a> AstReader<'a, DirectiveDefinitionId> {
    pub fn name(&self) -> &str {
        self.ast.lookup(self.ast.lookup(self.id).name)
    }

    pub fn description(&self) -> Option<&'a str> {
        self.ast
            .lookup(self.id)
            .description
            .map(|id| self.ast.lookup(id))
    }

    pub fn arguments(&self) -> impl Iterator<Item = AstReader<'a, InputValueDefinitionId>> {
        self.ast
            .lookup(self.id)
            .arguments
            .iter()
            .map(|id| self.ast.read(*id))
    }

    pub fn is_repeatable(&self) -> bool {
        self.ast.lookup(self.id).repeatable
    }

    pub fn locations(&self) -> impl Iterator<Item = DirectiveLocation> + 'a {
        self.ast.lookup(self.id).locations.iter().copied()
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

impl<'a> std::fmt::Display for AstReader<'a, TypeId> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Type { name, wrappers } = self.ast.lookup(self.id);

        for wrapping in wrappers.iter().rev() {
            if let WrappingType::List = wrapping {
                write!(f, "[")?;
            }
        }
        write!(f, "{}", self.ast.lookup(*name))?;
        for wrapping in wrappers.iter() {
            match wrapping {
                WrappingType::NonNull => write!(f, "!")?,
                WrappingType::List => write!(f, "]")?,
            }
        }

        Ok(())
    }
}
