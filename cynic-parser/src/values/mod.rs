mod const_lists;
mod const_objects;
mod const_value;
mod enums;
mod lists;
mod objects;
mod scalars;
mod value;
mod variables;

pub mod ids;
pub mod iter;
pub mod writer;

use std::sync::Arc;

pub use self::{
    const_lists::ConstList,
    const_objects::{ConstObject, ConstObjectField},
    const_value::ConstValue,
    enums::EnumValue,
    lists::List,
    objects::{Object, ObjectField},
    scalars::{BooleanValue, FloatValue, IntValue, NullValue, StringValue},
    value::Value,
    variables::VariableValue,
};

use ids::*;
use indexmap::IndexSet;

pub mod storage {
    pub use super::{
        objects::FieldRecord,
        value::{ValueKind, ValueRecord},
    };
}

#[derive(Default)]
pub struct ValueStore {
    strings: Arc<IndexSet<Box<str>>>,

    values: Vec<storage::ValueRecord>,

    fields: Vec<storage::FieldRecord>,
}

impl ValueStore {
    pub fn read<T>(&self, id: T) -> T::Reader<'_>
    where
        T: ValueStoreId,
    {
        id.read(self)
    }
}

#[derive(Clone, Copy)]
struct Cursor<'a, I> {
    id: I,
    store: &'a ValueStore,
}

pub trait ValueStoreId: Copy {
    type Reader<'a>;

    fn read(self, store: &ValueStore) -> Self::Reader<'_>;
}
