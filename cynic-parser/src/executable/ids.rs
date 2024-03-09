use std::num::NonZeroU32;

use super::{
    argument::ArgumentRecord, definition::ExecutableDefinitionRecord, directive::DirectiveRecord,
    storage::*, value::ValueRecord, Ast,
};
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

        impl AstLookup<$name> for Ast {
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

            pub fn iter(&self) -> impl Iterator<Item = $name> {
                (self.start.0.get()..self.end.0.get())
                    .map(|num| $name(NonZeroU32::new(num).expect("range is too large")))
            }
        }
    };
}

make_id!(
    ExecutableDefinitionId,
    ExecutableDefinitionRecord,
    definitions
);

make_id!(OperationDefinitionId, OperationDefinitionRecord, operations);

make_id!(FragmentDefinitionId, FragmentDefinitionRecord, fragments);

make_id!(SelectionId, SelectionRecord, selections);
impl_id_range!(SelectionId);

make_id!(FieldSelectionId, FieldSelectionRecord, field_selections);
make_id!(InlineFragmentId, InlineFragmentRecord, inline_fragments);
make_id!(FragmentSpreadId, FragmentSpreadRecord, fragment_spreads);

make_id!(DirectiveId, DirectiveRecord, directives);
make_id!(ArgumentId, ArgumentRecord, arguments);
impl_id_range!(DirectiveId);
impl_id_range!(ArgumentId);

make_id!(ValueId, ValueRecord, values);

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

impl AstLookup<StringId> for Ast {
    type Output = str;

    fn lookup(&self, index: StringId) -> &Self::Output {
        &self.strings[(index.0.get() - 1) as usize]
    }

    fn lookup_mut(&mut self, _index: StringId) -> &mut Self::Output {
        unimplemented!("strings aren't mutable so can't do this")
    }
}
