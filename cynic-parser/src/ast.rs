use std::collections::HashMap;

#[derive(Default)]
pub struct Ast {
    strings: HashMap<Box<str>, StringId>,

    nodes: Vec<Node>,

    definition_nodes: Vec<NodeId>,

    schema_definitions: Vec<SchemaDefinition>,
    object_definitions: Vec<ObjectDefinition>,
    input_object_definitions: Vec<InputObjectDefinition>,

    field_definitions: Vec<FieldDefinition>,
    input_value_definitions: Vec<InputValueDefinition>,

    string_literals: Vec<StringLiteral>,

    definition_descriptions: HashMap<NodeId, NodeId>,
}

// TODO: NonZeroUsize these?
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

#[derive(Clone, Copy)]
pub struct StringId(usize);

#[derive(Clone, Copy)]
pub struct SchemaDefinitionId(usize);

#[derive(Clone, Copy)]
pub struct ObjectDefinitionId(usize);

#[derive(Clone, Copy)]
pub struct InputObjectDefinitionId(usize);

#[derive(Clone, Copy)]
pub struct FieldDefinitionId(usize);

#[derive(Clone, Copy)]
pub struct InputValueDefinitionId(usize);

#[derive(Clone, Copy)]
pub struct StringLiteralId(usize);

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

#[derive(Debug)]
pub struct SchemaDefinition {
    pub roots: Vec<RootOperationTypeDefinition>,
}

#[derive(Debug)]
pub struct ObjectDefinition {
    pub name: NodeId,
    pub fields: Vec<NodeId>,
}

pub struct FieldDefinition {
    pub name: NodeId,
    pub ty: NodeId,
    pub arguments: Vec<NodeId>,
    pub description: Option<NodeId>,
}

#[derive(Debug)]
pub struct InputObjectDefinition {
    pub name: NodeId,
    pub fields: Vec<NodeId>,
}

pub struct InputValueDefinition {
    pub name: NodeId,
    pub ty: NodeId,
    pub description: Option<NodeId>,
}

#[derive(Debug)]
pub struct RootOperationTypeDefinition {
    pub operation_type: OperationType,
    pub named_type: NodeId,
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

// TODO: Don't forget the spans etc.

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

    pub fn string_literal(&mut self, literal: StringLiteral) -> NodeId {
        let literal_id = StringLiteralId(self.string_literals.len());
        self.string_literals.push(literal);

        let node_id = NodeId(self.nodes.len());
        let contents = NodeContents::StringLiteral(literal_id);
        self.nodes.push(Node { contents });

        node_id
    }

    pub fn ident(&mut self, ident: &str) -> NodeId {
        let id = NodeId(self.nodes.len());
        let contents = NodeContents::Ident(self.intern_string(ident));

        self.nodes.push(Node { contents });

        id
    }

    // TOOD: should this be pub? not sure...
    pub fn intern_string(&mut self, string: &str) -> StringId {
        if let Some(id) = self.strings.get(string) {
            return *id;
        }

        let id = StringId(self.strings.len());
        self.strings.insert(string.into(), id);
        id
    }
}
