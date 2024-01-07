use crate::Ast;

use super::{
    Argument, Directive, FieldDefinition, InputObjectDefinition, InputValueDefinition, Node,
    ObjectDefinition, SchemaDefinition, Type, Value,
};

pub trait AstId {}

pub(crate) trait AstLookup<Id> {
    type Output: ?Sized;

    fn lookup(&self, index: Id) -> &Self::Output;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub(super) usize);

impl AstLookup<NodeId> for Ast {
    type Output = Node;

    fn lookup(&self, index: NodeId) -> &Self::Output {
        &self.nodes[index.0]
    }
}

impl AstId for NodeId {}

#[derive(Clone, Copy)]
pub struct StringId(pub(super) usize);

impl AstLookup<StringId> for Ast {
    type Output = str;

    fn lookup(&self, index: StringId) -> &Self::Output {
        self.strings
            .get_index(index.0)
            .expect("strings to be present")
    }
}

#[derive(Clone, Copy)]
pub struct SchemaDefinitionId(pub(super) usize);

impl AstId for SchemaDefinitionId {}

impl AstLookup<SchemaDefinitionId> for Ast {
    type Output = SchemaDefinition;

    fn lookup(&self, index: SchemaDefinitionId) -> &Self::Output {
        self.schema_definitions
            .get(index.0)
            .expect("objects to be present")
    }
}

#[derive(Clone, Copy)]
pub struct ObjectDefinitionId(pub(super) usize);

impl AstId for ObjectDefinitionId {}

impl AstLookup<ObjectDefinitionId> for Ast {
    type Output = ObjectDefinition;

    fn lookup(&self, index: ObjectDefinitionId) -> &Self::Output {
        self.object_definitions
            .get(index.0)
            .expect("objects to be present")
    }
}

#[derive(Clone, Copy)]
pub struct InputObjectDefinitionId(pub(super) usize);
impl AstId for InputObjectDefinitionId {}

impl AstLookup<InputObjectDefinitionId> for Ast {
    type Output = InputObjectDefinition;

    fn lookup(&self, index: InputObjectDefinitionId) -> &Self::Output {
        self.input_object_definitions
            .get(index.0)
            .expect("objects to be present")
    }
}

#[derive(Clone, Copy)]
pub struct FieldDefinitionId(pub(super) usize);

impl AstId for FieldDefinitionId {}

impl AstLookup<FieldDefinitionId> for Ast {
    type Output = FieldDefinition;

    fn lookup(&self, index: FieldDefinitionId) -> &Self::Output {
        self.field_definitions
            .get(index.0)
            .expect("objects to be present")
    }
}

#[derive(Clone, Copy)]
pub struct InputValueDefinitionId(pub(super) usize);

impl AstId for InputValueDefinitionId {}

impl AstLookup<InputValueDefinitionId> for Ast {
    type Output = InputValueDefinition;

    fn lookup(&self, index: InputValueDefinitionId) -> &Self::Output {
        self.input_value_definitions
            .get(index.0)
            .expect("objects to be present")
    }
}

#[derive(Clone, Copy)]
pub struct TypeId(pub(super) usize);

impl AstId for TypeId {}

impl AstLookup<TypeId> for Ast {
    type Output = Type;

    fn lookup(&self, index: TypeId) -> &Self::Output {
        self.type_references
            .get(index.0)
            .expect("types to be present")
    }
}

#[derive(Clone, Copy)]
pub struct DirectiveId(pub(super) usize);

impl AstId for DirectiveId {}

impl AstLookup<DirectiveId> for Ast {
    type Output = Directive;

    fn lookup(&self, index: DirectiveId) -> &Self::Output {
        self.directives.get(index.0).expect("values to be present")
    }
}

#[derive(Clone, Copy)]
pub struct ArgumentId(pub(super) usize);

impl AstId for ArgumentId {}

impl AstLookup<ArgumentId> for Ast {
    type Output = Argument;

    fn lookup(&self, index: ArgumentId) -> &Self::Output {
        self.arguments.get(index.0).expect("values to be present")
    }
}

#[derive(Clone, Copy)]
pub struct ValueId(pub(super) usize);

impl AstId for ValueId {}

impl AstLookup<ValueId> for Ast {
    type Output = Value;

    fn lookup(&self, index: ValueId) -> &Self::Output {
        self.values.get(index.0).expect("values to be present")
    }
}

#[derive(Clone, Copy)]
pub struct StringLiteralId(pub(super) usize);
