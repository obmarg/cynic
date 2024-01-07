use std::ops::Index;

use crate::Ast;

use super::{FieldDefinition, InputObjectDefinition, InputValueDefinition, Node, ObjectDefinition};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub(super) usize);

impl Index<NodeId> for Ast {
    type Output = Node;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.nodes[index.0]
    }
}

#[derive(Clone, Copy)]
pub struct StringId(pub(super) usize);

impl Index<StringId> for Ast {
    type Output = str;

    fn index(&self, index: StringId) -> &Self::Output {
        self.strings
            .get_index(index.0)
            .expect("strings to be present")
    }
}

#[derive(Clone, Copy)]
pub struct SchemaDefinitionId(pub(super) usize);

#[derive(Clone, Copy)]
pub struct ObjectDefinitionId(pub(super) usize);

impl Index<ObjectDefinitionId> for Ast {
    type Output = ObjectDefinition;

    fn index(&self, index: ObjectDefinitionId) -> &Self::Output {
        self.object_definitions
            .get(index.0)
            .expect("objects to be present")
    }
}

#[derive(Clone, Copy)]
pub struct InputObjectDefinitionId(pub(super) usize);

impl Index<InputObjectDefinitionId> for Ast {
    type Output = InputObjectDefinition;

    fn index(&self, index: InputObjectDefinitionId) -> &Self::Output {
        self.input_object_definitions
            .get(index.0)
            .expect("objects to be present")
    }
}

#[derive(Clone, Copy)]
pub struct FieldDefinitionId(pub(super) usize);

impl Index<FieldDefinitionId> for Ast {
    type Output = FieldDefinition;

    fn index(&self, index: FieldDefinitionId) -> &Self::Output {
        self.field_definitions
            .get(index.0)
            .expect("objects to be present")
    }
}

#[derive(Clone, Copy)]
pub struct InputValueDefinitionId(pub(super) usize);

impl Index<InputValueDefinitionId> for Ast {
    type Output = InputValueDefinition;

    fn index(&self, index: InputValueDefinitionId) -> &Self::Output {
        self.input_value_definitions
            .get(index.0)
            .expect("objects to be present")
    }
}

#[derive(Clone, Copy)]
pub struct StringLiteralId(pub(super) usize);
