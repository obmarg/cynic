use std::collections::HashMap;

pub struct Ast {
    strings: HashMap<Box<str>, StringId>,

    nodes: Vec<Node>,
    schema_definitions: Vec<SchemaDefinition>,
    object_definitions: Vec<ObjectDefinition>,

    field_definitions: Vec<FieldDefinition>,
}

// TODO: NonZeroUsize these?
#[derive(Clone, Copy, Debug)]
pub struct NodeId(usize);

#[derive(Clone, Copy)]
pub struct StringId(usize);

#[derive(Clone, Copy)]
pub struct SchemaDefinitionId(usize);

#[derive(Clone, Copy)]
pub struct FieldDefinitionId(usize);

pub struct Node {
    contents: NodeContents,
    // span: Span
}

pub enum NodeContents {
    Ident(StringId),
    SchemaDefinition(SchemaDefinitionId),
    FieldDefinition(FieldDefinitionId),
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

// TODO: Don't forget the spans etc.

impl Ast {
    pub fn new() -> Self {
        Ast {
            strings: Default::default(),
            nodes: Default::default(),
            schema_definitions: Default::default(),
            object_definitions: Default::default(),
            field_definitions: Default::default(),
        }
    }

    pub fn schema_definition(&mut self, roots: Vec<RootOperationTypeDefinition>) -> NodeId {
        let definition_id = SchemaDefinitionId(self.schema_definitions.len());
        self.schema_definitions.push(SchemaDefinition { roots });

        let node_id = NodeId(self.nodes.len());
        let contents = NodeContents::SchemaDefinition(definition_id);

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

    pub fn ident(&mut self, ident: &str) -> NodeId {
        let id = NodeId(self.nodes.len());
        let contents = NodeContents::Ident(self.intern_string(ident));

        self.nodes.push(Node { contents });

        id
    }

    fn intern_string(&mut self, string: &str) -> StringId {
        if let Some(id) = self.strings.get(string) {
            return *id;
        }

        let id = StringId(self.strings.len());
        self.strings.insert(string.into(), id);
        id
    }
}
