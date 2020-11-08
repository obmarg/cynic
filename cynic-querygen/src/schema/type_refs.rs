use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use std::{convert::TryInto, fmt, rc::Rc};

use super::{InputType, OutputType, TypeIndex};
use crate::Error;

#[derive(Clone)]
pub struct InputTypeRef<'schema> {
    pub(super) type_name: &'schema str,
    type_index: Rc<TypeIndex<'schema>>,
}

#[derive(Clone)]
pub struct OutputTypeRef<'schema> {
    pub(super) type_name: &'schema str,
    type_index: Rc<TypeIndex<'schema>>,
}

macro_rules! impl_type_ref {
    ($target:ident, $lookup_type:ident) => {
        impl<'schema> $target<'schema> {
            pub(super) fn new(
                type_name: &'schema str,
                type_index: &Rc<TypeIndex<'schema>>,
            ) -> Self {
                Self {
                    type_name,
                    type_index: Rc::clone(type_index),
                }
            }

            pub fn lookup(&self) -> Result<$lookup_type<'schema>, Error> {
                self.type_index.lookup_type_2(self.type_name)?.try_into()
            }
        }

        impl<'schema> PartialEq for $target<'schema> {
            fn eq(&self, other: &Self) -> bool {
                self.type_name == other.type_name
            }
        }

        impl<'schema> Eq for $target<'schema> {}

        impl<'schema> PartialOrd for $target<'schema> {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.type_name.cmp(other.type_name))
            }
        }

        impl<'schema> Ord for $target<'schema> {
            fn cmp(&self, other: &Self) -> Ordering {
                self.type_name.cmp(other.type_name)
            }
        }

        impl<'schema> Hash for $target<'schema> {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.type_name.hash(state);
            }
        }

        impl<'schema> fmt::Debug for $target<'schema> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($target))
                    .field("type_name", &self.type_name)
                    .finish()
            }
        }
    };
}

impl_type_ref!(InputTypeRef, InputType);
impl_type_ref!(OutputTypeRef, OutputType);
