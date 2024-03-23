use std::num::NonZeroU32;

use super::{storage::*, DefinitionRecord, TypeSystemDocument};
use crate::{common::IdRange, AstLookup};

macro_rules! make_id {
    ($name:ident, $output:ident, $field:ident) => {
        #[derive(Clone, Copy)]
        pub struct $name(NonZeroU32);

        impl $name {
            pub(super) fn new(index: usize) -> Self {
                Self(
                    NonZeroU32::new(u32::try_from(index + 1).expect("too many indices"))
                        .expect("also too many indices"),
                )
            }
        }

        impl AstLookup<$name> for TypeSystemDocument {
            type Output = $output;

            fn lookup(&self, index: $name) -> &Self::Output {
                &self.$field[(index.0.get() - 1) as usize]
            }

            fn lookup_mut(&mut self, index: $name) -> &mut Self::Output {
                &mut self.$field[(index.0.get() - 1) as usize]
            }
        }
    };
}

macro_rules! impl_id_range {
    ($name: ident) => {
        impl IdRange<$name> {
            pub fn len(&self) -> usize {
                (self.end.0.get() - self.start.0.get()) as usize
            }

            pub fn is_empty(&self) -> bool {
                (self.end.0.get() - self.start.0.get()) == 0
            }

            pub fn iter(&self) -> impl ExactSizeIterator<Item = $name> {
                (self.start.0.get()..self.end.0.get())
                    .map(|num| $name(NonZeroU32::new(num).expect("range is too large")))
            }
        }

        impl Default for IdRange<$name> {
            fn default() -> Self {
                Self::new($name::new(0), $name::new(0))
            }
        }
    };
}

make_id!(DefinitionId, DefinitionRecord, definitions);

make_id!(
    SchemaDefinitionId,
    SchemaDefinitionRecord,
    schema_definitions
);
make_id!(
    RootOperationTypeDefinitionId,
    RootOperationTypeDefinitionRecord,
    root_operation_definitions
);
impl_id_range!(RootOperationTypeDefinitionId);

make_id!(
    ScalarDefinitionId,
    ScalarDefinitionRecord,
    scalar_definitions
);

make_id!(
    ObjectDefinitionId,
    ObjectDefinitionRecord,
    object_definitions
);

make_id!(
    InterfaceDefinitionId,
    InterfaceDefinitionRecord,
    interface_definitions
);

make_id!(UnionDefinitionId, UnionDefinitionRecord, union_definitions);

make_id!(EnumDefinitionId, EnumDefinitionRecord, enum_definitions);

make_id!(
    EnumValueDefinitionId,
    EnumValueDefinitionRecord,
    enum_value_definitions
);
impl_id_range!(EnumValueDefinitionId);

make_id!(
    InputObjectDefinitionId,
    InputObjectDefinitionRecord,
    input_object_definitions
);

make_id!(
    DirectiveDefinitionId,
    DirectiveDefinitionRecord,
    directive_definitions
);

make_id!(FieldDefinitionId, FieldDefinitionRecord, field_definitions);
impl_id_range!(FieldDefinitionId);

make_id!(
    InputValueDefinitionId,
    InputValueDefinitionRecord,
    input_value_definitions
);
impl_id_range!(InputValueDefinitionId);

make_id!(TypeId, TypeRecord, type_references);

make_id!(DirectiveId, DirectiveRecord, directives);
impl_id_range!(DirectiveId);

make_id!(ArgumentId, ArgumentRecord, arguments);
impl_id_range!(ArgumentId);

make_id!(ValueId, ValueRecord, values);

make_id!(BlockStringLiteralId, str, block_strings);

#[derive(Clone, Copy)]
pub struct StringId(NonZeroU32);

impl StringId {
    pub(super) fn new(index: usize) -> Self {
        Self(
            NonZeroU32::new(u32::try_from(index + 1).expect("too many indices"))
                .expect("also too many indices"),
        )
    }
}

impl AstLookup<StringId> for TypeSystemDocument {
    type Output = str;

    fn lookup(&self, index: StringId) -> &Self::Output {
        &self.strings[(index.0.get() - 1) as usize]
    }

    fn lookup_mut(&mut self, _index: StringId) -> &mut Self::Output {
        unimplemented!("strings aren't mutable so can't do this")
    }
}

#[derive(Clone, Copy)]
pub enum StringLiteralId {
    String(StringId),
    Block(BlockStringLiteralId),
}
