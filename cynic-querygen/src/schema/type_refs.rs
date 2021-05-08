use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use std::{borrow::Cow, convert::TryInto, fmt, rc::Rc};

use super::{InputType, InterfaceType, OutputType, Type, TypeIndex};
use crate::Error;

/// A reference to a `Type` in the `TypeIndex`.
#[derive(Clone)]
pub struct TypeRef<'schema> {
    pub(super) type_name: Cow<'schema, str>,
    type_index: Rc<TypeIndex<'schema>>,
}

/// A reference to an `InputType` in the `TypeIndex`.
#[derive(Clone)]
pub struct InputTypeRef<'schema> {
    pub(super) type_name: Cow<'schema, str>,
    type_index: Rc<TypeIndex<'schema>>,
}

/// A reference to an `OutputType` in the `TypeIndex`.
#[derive(Clone)]
pub struct OutputTypeRef<'schema> {
    pub(super) type_name: Cow<'schema, str>,
    type_index: Rc<TypeIndex<'schema>>,
}

/// A reference to an `Interface` in the `TypeIndex`.
#[derive(Clone)]
pub struct InterfaceTypeRef<'schema> {
    pub(super) type_name: Cow<'schema, str>,
    type_index: Rc<TypeIndex<'schema>>,
}

macro_rules! impl_type_ref {
    ($target:ident, $lookup_type:ident) => {
        impl<'schema> $target<'schema> {
            #[allow(dead_code)]
            pub(super) fn new(
                type_name: &'schema str,
                type_index: &Rc<TypeIndex<'schema>>,
            ) -> Self {
                Self {
                    type_name: Cow::Borrowed(type_name),
                    type_index: Rc::clone(type_index),
                }
            }

            #[allow(dead_code)]
            pub(super) fn new_owned(
                type_name: String,
                type_index: &Rc<TypeIndex<'schema>>,
            ) -> Self {
                Self {
                    type_name: Cow::Owned(type_name),
                    type_index: Rc::clone(type_index),
                }
            }

            #[cfg(test)]
            #[allow(dead_code)]
            pub fn test_ref(type_name: String, type_index: &Rc<TypeIndex<'schema>>) -> Self {
                Self {
                    type_name: Cow::Owned(type_name),
                    type_index: Rc::clone(type_index),
                }
            }

            #[allow(dead_code, clippy::needless_question_mark)]
            pub fn lookup(&self) -> Result<$lookup_type<'schema>, Error> {
                Ok(self.type_index.lookup_type(&self.type_name)?.try_into()?)
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
                Some(self.type_name.cmp(&other.type_name))
            }
        }

        impl<'schema> Ord for $target<'schema> {
            fn cmp(&self, other: &Self) -> Ordering {
                self.type_name.cmp(&other.type_name)
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
impl_type_ref!(InterfaceTypeRef, InterfaceType);
impl_type_ref!(TypeRef, Type);

/// We need this conversion for the `try_into?` for Type -> Type in TypeRef
/// impl to work.  The conversion can't actually fail so we shouldn't ever
/// hit this code - just to keep the compiler happy.
impl From<std::convert::Infallible> for Error {
    fn from(_: std::convert::Infallible) -> Error {
        panic!("this can't happen");
    }
}

macro_rules! impl_into_type_ref {
    ($target:ident) => {
        impl<'schema> From<$target<'schema>> for TypeRef<'schema> {
            fn from(other: $target<'schema>) -> TypeRef<'schema> {
                TypeRef {
                    type_name: other.type_name,
                    type_index: other.type_index,
                }
            }
        }
    };
}

impl_into_type_ref!(OutputTypeRef);
impl_into_type_ref!(InputTypeRef);
