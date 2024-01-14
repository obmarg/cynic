use crate::Ast;

use super::{
    storage::{
        Argument, Directive, DirectiveDefinition, EnumDefinition, EnumValueDefinition,
        FieldDefinition, InputObjectDefinition, InputValueDefinition, InterfaceDefinition,
        ObjectDefinition, ScalarDefinition, SchemaDefinition, Type, UnionDefinition, Value,
    },
    AstDefinition, AstLookup,
};

macro_rules! make_id {
    ($name:ident, $output:ident, $field:ident) => {
        #[derive(Clone, Copy)]
        pub struct $name(pub(super) usize);

        impl AstLookup<$name> for Ast {
            type Output = $output;

            fn lookup(&self, index: $name) -> &Self::Output {
                &self.$field[index.0]
            }
        }
    };
}

make_id!(DefinitionId, AstDefinition, definitions);
make_id!(StringId, str, strings);
make_id!(SchemaDefinitionId, SchemaDefinition, schema_definitions);
make_id!(ScalarDefinitionId, ScalarDefinition, scalar_definitions);
make_id!(ObjectDefinitionId, ObjectDefinition, object_definitions);
make_id!(
    InterfaceDefinitionId,
    InterfaceDefinition,
    interface_definitions
);
make_id!(UnionDefinitionId, UnionDefinition, union_definitions);
make_id!(EnumDefinitionId, EnumDefinition, enum_definitions);
make_id!(
    EnumValueDefinitionId,
    EnumValueDefinition,
    enum_value_definitions
);
make_id!(
    InputObjectDefinitionId,
    InputObjectDefinition,
    input_object_definitions
);
make_id!(
    DirectiveDefinitionId,
    DirectiveDefinition,
    directive_definitions
);
make_id!(FieldDefinitionId, FieldDefinition, field_definitions);
make_id!(
    InputValueDefinitionId,
    InputValueDefinition,
    input_value_definitions
);
make_id!(TypeId, Type, type_references);
make_id!(DirectiveId, Directive, directives);
make_id!(ArgumentId, Argument, arguments);
make_id!(ValueId, Value, values);

#[derive(Clone, Copy)]
pub struct StringLiteralId(pub(super) usize);
