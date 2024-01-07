use std::collections::HashMap;

use ids::*;
use indexmap::IndexSet;

pub(crate) mod ids;
mod reader;

pub use reader::{AstReader, Definition, ValueReader};

#[derive(Default)]
pub struct Ast {
    strings: IndexSet<Box<str>>,

    nodes: Vec<Node>,

    definition_nodes: Vec<NodeId>,

    schema_definitions: Vec<SchemaDefinition>,
    object_definitions: Vec<ObjectDefinition>,
    input_object_definitions: Vec<InputObjectDefinition>,

    field_definitions: Vec<FieldDefinition>,
    input_value_definitions: Vec<InputValueDefinition>,

    type_references: Vec<Type>,

    string_literals: Vec<StringLiteral>,

    values: Vec<Value>,
    directives: Vec<Directive>,
    arguments: Vec<Argument>,

    definition_descriptions: HashMap<NodeId, NodeId>,
}

// TODO: NonZeroUsize these?
pub struct Node {
    contents: NodeContents,
    // span: Span
}

pub enum NodeContents {
    Ident(StringId),
    SchemaDefinition(SchemaDefinitionId),
    ObjectDefiniton(ObjectDefinitionId),
    FieldDefinition(FieldDefinitionId),
    InputObjectDefiniton(InputObjectDefinitionId),
    InputValueDefinition(InputValueDefinitionId),
    StringLiteral(StringLiteralId),
}

pub struct SchemaDefinition {
    pub roots: Vec<RootOperationTypeDefinition>,
}

pub struct ObjectDefinition {
    pub name: StringId,
    pub fields: Vec<NodeId>,
    pub directives: Vec<DirectiveId>,
}

pub struct FieldDefinition {
    pub name: StringId,
    pub ty: TypeId,
    pub arguments: Vec<NodeId>,
    pub description: Option<NodeId>,
}

pub struct InputObjectDefinition {
    pub name: StringId,
    pub fields: Vec<NodeId>,
    pub directives: Vec<DirectiveId>,
}

pub struct InputValueDefinition {
    pub name: StringId,
    pub ty: TypeId,
    pub description: Option<NodeId>,
    pub default: Option<ValueId>,
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

#[derive(Debug)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
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

// TODO: Don't forget the spans etc.
// TODO: make this whole impl into a builder that wraps an Ast.
// Then the default Reader stuff can just go on Ast - much more sensible...
impl Ast {
    pub fn new() -> Self {
        Ast::default()
    }

    pub fn definitions(&mut self, ids: Vec<(Option<NodeId>, NodeId)>) {
        for (description, definition) in ids {
            if let Some(description) = description {
                self.definition_descriptions.insert(definition, description);
            }
            self.definition_nodes.push(definition);
        }
    }

    pub fn schema_definition(&mut self, definition: SchemaDefinition) -> NodeId {
        let definition_id = SchemaDefinitionId(self.schema_definitions.len());
        self.schema_definitions.push(definition);

        let node_id = NodeId(self.nodes.len());
        let contents = NodeContents::SchemaDefinition(definition_id);

        self.nodes.push(Node { contents });

        node_id
    }

    pub fn object_definition(&mut self, definition: ObjectDefinition) -> NodeId {
        let definition_id = ObjectDefinitionId(self.object_definitions.len());
        self.object_definitions.push(definition);

        let node_id = NodeId(self.nodes.len());
        let contents = NodeContents::ObjectDefiniton(definition_id);

        self.nodes.push(Node { contents });

        node_id
    }

    pub fn field_definition(&mut self, definition: FieldDefinition) -> NodeId {
        let definition_id = FieldDefinitionId(self.field_definitions.len());
        self.field_definitions.push(definition);

        let node_id = NodeId(self.nodes.len());
        let contents = NodeContents::FieldDefinition(definition_id);

        self.nodes.push(Node { contents });

        node_id
    }

    pub fn input_object_definition(&mut self, definition: InputObjectDefinition) -> NodeId {
        let definition_id = InputObjectDefinitionId(self.input_object_definitions.len());
        self.input_object_definitions.push(definition);

        let node_id = NodeId(self.nodes.len());
        let contents = NodeContents::InputObjectDefiniton(definition_id);

        self.nodes.push(Node { contents });

        node_id
    }

    pub fn input_value_definition(&mut self, definition: InputValueDefinition) -> NodeId {
        let definition_id = InputValueDefinitionId(self.input_value_definitions.len());
        self.input_value_definitions.push(definition);

        let node_id = NodeId(self.nodes.len());
        let contents = NodeContents::InputValueDefinition(definition_id);
        self.nodes.push(Node { contents });

        node_id
    }

    pub fn type_reference(&mut self, ty: Type) -> TypeId {
        let ty_id = TypeId(self.type_references.len());
        self.type_references.push(ty);
        ty_id
    }

    pub fn directive(&mut self, directive: Directive) -> DirectiveId {
        let id = DirectiveId(self.directives.len());
        self.directives.push(directive);
        id
    }

    pub fn argument(&mut self, argument: Argument) -> ArgumentId {
        let id = ArgumentId(self.arguments.len());
        self.arguments.push(argument);
        id
    }

    pub fn value(&mut self, value: Value) -> ValueId {
        let id = ValueId(self.values.len());
        self.values.push(value);
        id
    }

    pub fn string_literal(&mut self, literal: StringLiteral) -> NodeId {
        let literal_id = StringLiteralId(self.string_literals.len());
        self.string_literals.push(literal);

        let node_id = NodeId(self.nodes.len());
        let contents = NodeContents::StringLiteral(literal_id);
        self.nodes.push(Node { contents });

        node_id
    }

    pub fn ident(&mut self, ident: &str) -> StringId {
        self.intern_string(ident)
    }

    // TOOD: should this be pub? not sure...
    pub fn intern_string(&mut self, string: &str) -> StringId {
        let (id, _) = self.strings.insert_full(string.into());
        StringId(id)
    }
}
