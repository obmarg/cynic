use std::collections::HashMap;

use ids::*;
use indexmap::IndexSet;

pub(crate) mod ids;
mod reader;
mod span;

pub use reader::{AstReader, Definition, ValueReader};
pub use span::Span;

#[derive(Default)]
pub struct Ast {
    strings: IndexSet<Box<str>>,

    definitions: Vec<AstDefinition>,

    schema_definitions: Vec<SchemaDefinition>,
    object_definitions: Vec<ObjectDefinition>,
    interface_definitions: Vec<InterfaceDefinition>,
    input_object_definitions: Vec<InputObjectDefinition>,

    field_definitions: Vec<FieldDefinition>,
    input_value_definitions: Vec<InputValueDefinition>,

    type_references: Vec<Type>,

    string_literals: Vec<StringLiteral>,

    values: Vec<Value>,
    directives: Vec<Directive>,
    arguments: Vec<Argument>,

    definition_descriptions: HashMap<DefinitionId, StringId>,
}

// TODO: Do something about the name clash
pub enum AstDefinition {
    Schema(SchemaDefinitionId),
    Object(ObjectDefinitionId),
    Interface(InterfaceDefinitionId),
    InputObject(InputObjectDefinitionId),
}

pub struct SchemaDefinition {
    pub roots: Vec<RootOperationTypeDefinition>,
}

pub struct ObjectDefinition {
    pub name: StringId,
    pub fields: Vec<FieldDefinitionId>,
    pub directives: Vec<DirectiveId>,
    pub implements: Vec<StringId>,
    pub span: Span,
}

pub struct FieldDefinition {
    pub name: StringId,
    pub ty: TypeId,
    pub arguments: Vec<InputValueDefinitionId>,
    pub description: Option<StringId>,
    pub directives: Vec<DirectiveId>,
    pub span: Span,
}

pub struct InterfaceDefinition {
    pub name: StringId,
    pub fields: Vec<FieldDefinitionId>,
    pub directives: Vec<DirectiveId>,
    pub implements: Vec<StringId>,
    pub span: Span,
}

pub struct InputObjectDefinition {
    pub name: StringId,
    pub fields: Vec<InputValueDefinitionId>,
    pub directives: Vec<DirectiveId>,
    pub span: Span,
}

pub struct InputValueDefinition {
    pub name: StringId,
    pub ty: TypeId,
    pub description: Option<StringId>,
    pub default: Option<ValueId>,
    pub directives: Vec<DirectiveId>,
    pub span: Span,
}

pub struct RootOperationTypeDefinition {
    pub operation_type: OperationType,
    pub named_type: StringId,
}

pub struct Type {
    pub name: StringId,
    pub wrappers: Vec<WrappingType>,
}

pub enum WrappingType {
    NonNull,
    List,
}

#[derive(Clone, Copy, Debug)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::Query => write!(f, "query"),
            OperationType::Mutation => write!(f, "mutation"),
            OperationType::Subscription => write!(f, "subscription"),
        }
    }
}

pub enum StringLiteral {
    Normal(StringId),
    Block(StringId),
}

pub struct Directive {
    pub name: StringId,
    pub arguments: Vec<ArgumentId>,
}

pub struct Argument {
    pub name: StringId,
    pub value: ValueId,
}

pub enum Value {
    Variable(StringId),
    Int(i32),
    Float(f32),
    String(StringId),
    Boolean(bool),
    Null,
    Enum(StringId),
    List(Vec<ValueId>),
    Object(Vec<(StringId, ValueId)>),
}

pub struct AstBuilder {
    ast: Ast,
}

// TODO: Don't forget the spans etc.
impl AstBuilder {
    pub fn new() -> Self {
        AstBuilder {
            ast: Default::default(),
        }
    }

    pub fn finish(self) -> Ast {
        self.ast
    }

    pub fn definition_descriptions(&mut self, ids: Vec<(Option<StringId>, DefinitionId)>) {
        for (description, definition) in ids {
            if let Some(description) = description {
                self.ast
                    .definition_descriptions
                    .insert(definition, description);
            }
        }
    }

    pub fn schema_definition(&mut self, definition: SchemaDefinition) -> DefinitionId {
        let id = SchemaDefinitionId(self.ast.schema_definitions.len());
        self.ast.schema_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::Schema(id));

        definition_id
    }

    pub fn object_definition(&mut self, definition: ObjectDefinition) -> DefinitionId {
        let id = ObjectDefinitionId(self.ast.object_definitions.len());
        self.ast.object_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::Object(id));

        definition_id
    }

    pub fn field_definition(&mut self, definition: FieldDefinition) -> FieldDefinitionId {
        let definition_id = FieldDefinitionId(self.ast.field_definitions.len());
        self.ast.field_definitions.push(definition);

        definition_id
    }

    pub fn interface_definition(&mut self, definition: InterfaceDefinition) -> DefinitionId {
        let id = InterfaceDefinitionId(self.ast.interface_definitions.len());
        self.ast.interface_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::Interface(id));

        definition_id
    }

    pub fn input_object_definition(&mut self, definition: InputObjectDefinition) -> DefinitionId {
        let id = InputObjectDefinitionId(self.ast.input_object_definitions.len());
        self.ast.input_object_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::InputObject(id));

        definition_id
    }

    pub fn input_value_definition(
        &mut self,
        definition: InputValueDefinition,
    ) -> InputValueDefinitionId {
        let definition_id = InputValueDefinitionId(self.ast.input_value_definitions.len());
        self.ast.input_value_definitions.push(definition);

        definition_id
    }

    pub fn type_reference(&mut self, ty: Type) -> TypeId {
        let ty_id = TypeId(self.ast.type_references.len());
        self.ast.type_references.push(ty);
        ty_id
    }

    pub fn directive(&mut self, directive: Directive) -> DirectiveId {
        let id = DirectiveId(self.ast.directives.len());
        self.ast.directives.push(directive);
        id
    }

    pub fn argument(&mut self, argument: Argument) -> ArgumentId {
        let id = ArgumentId(self.ast.arguments.len());
        self.ast.arguments.push(argument);
        id
    }

    pub fn value(&mut self, value: Value) -> ValueId {
        let id = ValueId(self.ast.values.len());
        self.ast.values.push(value);
        id
    }

    pub fn string_literal(&mut self, literal: StringLiteral) -> StringLiteralId {
        let literal_id = StringLiteralId(self.ast.string_literals.len());
        self.ast.string_literals.push(literal);

        literal_id
    }

    pub fn ident(&mut self, ident: &str) -> StringId {
        self.intern_string(ident)
    }

    // TOOD: should this be pub? not sure...
    pub fn intern_string(&mut self, string: &str) -> StringId {
        let (id, _) = self.ast.strings.insert_full(string.into());
        StringId(id)
    }
}
