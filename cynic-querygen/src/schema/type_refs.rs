use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use std::{
    convert::{TryFrom, TryInto},
    rc::Rc,
};

use super::{InputType, OutputType, Type, TypeIndex};
use crate::Error;

pub struct InputTypeRef<'schema> {
    type_name: &'schema str,
    type_index: Rc<TypeIndex<'schema>>,
}

pub struct OutputTypeRef<'schema> {
    type_name: &'schema str,
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
    };
}

impl_type_ref!(InputTypeRef, InputType);
impl_type_ref!(OutputTypeRef, OutputType);

impl<'schema> TryFrom<super::Type<'schema>> for InputType<'schema> {
    type Error = Error;

    fn try_from(ty: Type<'schema>) -> Result<InputType<'schema>, Error> {
        match ty {
            Type::InputObject(inner) => Ok(InputType::InputObject(inner)),
            Type::Scalar(inner) => Ok(InputType::Scalar(inner)),
            Type::Enum(inner) => Ok(InputType::Enum(inner)),
            _ => Err(Error::ExpectedInputType),
        }
    }
}

impl<'schema> TryFrom<Type<'schema>> for OutputType<'schema> {
    type Error = Error;

    fn try_from(ty: Type<'schema>) -> Result<OutputType<'schema>, Error> {
        match ty {
            Type::Scalar(inner) => Ok(OutputType::Scalar(inner)),
            Type::Enum(inner) => Ok(OutputType::Enum(inner)),
            Type::Object(inner) => Ok(OutputType::Object(inner)),
            Type::Interface(inner) => Ok(OutputType::Interface(inner)),
            Type::Union(inner) => Ok(OutputType::Union(inner)),
            Type::InputObject(_) => Err(Error::ExpectedOutputType),
        }
    }
}
