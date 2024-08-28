use std::num::NonZeroU32;

use super::{storage::*, DefinitionRecord, TypeSystemDocument};
use crate::{common::IdRange, AstLookup};

macro_rules! make_id {
    ($name:ident, $output:ident, $field:ident) => {
        #[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
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

macro_rules! impl_id_range_ops {
    ($name: ident) => {
        impl crate::common::IdOperations for $name {
            fn empty_range() -> IdRange<Self> {
                IdRange::new(Self::new(0), Self::new(0))
            }
            fn forward(self) -> Option<Self> {
                Some(Self(NonZeroU32::new(self.0.get() + 1)?))
            }
            fn back(self) -> Option<Self> {
                Some(Self(NonZeroU32::new(self.0.get() - 1)?))
            }
            fn cmp(self, other: Self) -> std::cmp::Ordering {
                self.0.get().cmp(&other.0.get())
            }
            fn distance(lhs: Self, rhs: Self) -> usize {
                rhs.0.get().saturating_sub(lhs.0.get()) as usize
            }
        }
    };
}

make_id!(DefinitionId, DefinitionRecord, definitions);
impl_id_range_ops!(DefinitionId);

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
impl_id_range_ops!(RootOperationTypeDefinitionId);

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

make_id!(UnionMemberId, UnionMemberRecord, union_members);
impl_id_range_ops!(UnionMemberId);

make_id!(EnumDefinitionId, EnumDefinitionRecord, enum_definitions);

make_id!(
    EnumValueDefinitionId,
    EnumValueDefinitionRecord,
    enum_value_definitions
);
impl_id_range_ops!(EnumValueDefinitionId);

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
impl_id_range_ops!(FieldDefinitionId);

make_id!(
    InputValueDefinitionId,
    InputValueDefinitionRecord,
    input_value_definitions
);
impl_id_range_ops!(InputValueDefinitionId);

make_id!(TypeId, TypeRecord, type_references);

make_id!(DirectiveId, DirectiveRecord, directives);
impl_id_range_ops!(DirectiveId);

make_id!(ArgumentId, ArgumentRecord, arguments);
impl_id_range_ops!(ArgumentId);

make_id!(ValueId, ValueRecord, values);

make_id!(DescriptionId, DescriptionRecord, descriptions);

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
