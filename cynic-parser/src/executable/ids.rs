use std::num::NonZeroU32;

use super::{storage::*, types::TypeRecord, value::ValueRecord, ExecutableDocument};
use crate::{common::IdRange, AstLookup};

macro_rules! make_id {
    ($name:ident, $output:ident, $field:ident) => {
        #[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
        pub struct $name(NonZeroU32);

        impl $name {
            pub(super) fn new(index: usize) -> Self {
                Self(
                    NonZeroU32::new(u32::try_from(index + 1).expect("too many indices"))
                        .expect("also too many indices"),
                )
            }
        }

        impl AstLookup<$name> for ExecutableDocument {
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

make_id!(
    ExecutableDefinitionId,
    ExecutableDefinitionRecord,
    definitions
);
impl_id_range_ops!(ExecutableDefinitionId);

make_id!(OperationDefinitionId, OperationDefinitionRecord, operations);

make_id!(FragmentDefinitionId, FragmentDefinitionRecord, fragments);

make_id!(SelectionId, SelectionRecord, selections);
impl_id_range_ops!(SelectionId);

make_id!(FieldSelectionId, FieldSelectionRecord, field_selections);
make_id!(InlineFragmentId, InlineFragmentRecord, inline_fragments);
make_id!(FragmentSpreadId, FragmentSpreadRecord, fragment_spreads);

make_id!(DirectiveId, DirectiveRecord, directives);
make_id!(ArgumentId, ArgumentRecord, arguments);
impl_id_range_ops!(DirectiveId);
impl_id_range_ops!(ArgumentId);

make_id!(TypeId, TypeRecord, types);

make_id!(VariableDefinitionId, VariableDefinitionRecord, variables);
impl_id_range_ops!(VariableDefinitionId);

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

impl AstLookup<StringId> for ExecutableDocument {
    type Output = str;

    fn lookup(&self, index: StringId) -> &Self::Output {
        &self.strings[(index.0.get() - 1) as usize]
    }

    fn lookup_mut(&mut self, _index: StringId) -> &mut Self::Output {
        unimplemented!("strings aren't mutable so can't do this")
    }
}
