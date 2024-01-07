use crate::Ast;

use super::{
    Argument, AstDefinition, Directive, EnumDefinition, EnumValueDefinition, FieldDefinition,
    InputObjectDefinition, InputValueDefinition, InterfaceDefinition, ObjectDefinition,
    SchemaDefinition, Type, UnionDefinition, Value,
};

pub trait AstId {}

pub(crate) trait AstLookup<Id> {
    type Output: ?Sized;

    fn lookup(&self, index: Id) -> &Self::Output;
}

// TODO: NonZeroUsize these?

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DefinitionId(pub(super) usize);

impl AstLookup<DefinitionId> for Ast {
    type Output = AstDefinition;

    fn lookup(&self, index: DefinitionId) -> &Self::Output {
        &self.definitions[index.0]
    }
}

impl AstId for DefinitionId {}

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
pub struct InterfaceDefinitionId(pub(super) usize);

impl AstId for InterfaceDefinitionId {}

impl AstLookup<InterfaceDefinitionId> for Ast {
    type Output = InterfaceDefinition;

    fn lookup(&self, index: InterfaceDefinitionId) -> &Self::Output {
        self.interface_definitions
            .get(index.0)
            .expect("objects to be present")
    }
}

#[derive(Clone, Copy)]
pub struct UnionDefinitionId(pub(super) usize);

impl AstId for UnionDefinitionId {}

impl AstLookup<UnionDefinitionId> for Ast {
    type Output = UnionDefinition;

    fn lookup(&self, index: UnionDefinitionId) -> &Self::Output {
        self.union_definitions
            .get(index.0)
            .expect("objects to be present")
    }
}

#[derive(Clone, Copy)]
pub struct EnumDefinitionId(pub(super) usize);

impl AstId for EnumDefinitionId {}

impl AstLookup<EnumDefinitionId> for Ast {
    type Output = EnumDefinition;

    fn lookup(&self, index: EnumDefinitionId) -> &Self::Output {
        self.enum_definitions
            .get(index.0)
            .expect("objects to be present")
    }
}

#[derive(Clone, Copy)]
pub struct EnumValueDefinitionId(pub(super) usize);

impl AstId for EnumValueDefinitionId {}

impl AstLookup<EnumValueDefinitionId> for Ast {
    type Output = EnumValueDefinition;

    fn lookup(&self, index: EnumValueDefinitionId) -> &Self::Output {
        self.enum_value_definitions
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
