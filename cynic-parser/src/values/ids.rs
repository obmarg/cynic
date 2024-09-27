use std::num::NonZeroU32;

use super::{storage::*, ValueStore};
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

        impl AstLookup<$name> for ValueStore {
            type Output = $output;

            fn lookup(&self, index: $name) -> &Self::Output {
                &self.$field[(index.0.get() - 1) as usize]
            }
        }
    };
}

make_id!(ValueId, ValueRecord, values);
make_id!(FieldId, FieldRecord, fields);

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

impl_id_range_ops!(ValueId);
impl_id_range_ops!(FieldId);

#[derive(Clone, Copy)]
pub struct StringId(NonZeroU32);

impl StringId {
    pub(super) fn new(index: usize) -> Self {
        Self(
            NonZeroU32::new(u32::try_from(index + 1).expect("too many indices"))
                .expect("also too many indices"),
        )
    }

    pub(crate) fn from_executable_id(id: crate::executable::ids::StringId) -> Self {
        Self::new(id.get())
    }

    pub(crate) fn from_type_system_id(id: crate::type_system::ids::StringId) -> Self {
        Self::new(id.get())
    }
}

impl AstLookup<StringId> for ValueStore {
    type Output = str;

    fn lookup(&self, index: StringId) -> &Self::Output {
        &self.strings[(index.0.get() - 1) as usize]
    }
}
