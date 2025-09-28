use std::fmt;

use crate::{AstLookup, Span};

use super::{
    Cursor, ValueStoreId,
    const_objects::ConstObject,
    ids::{FieldId, StringId, ValueId},
    iter::{Iter, ValueStoreReader},
    value::Value,
};

pub struct FieldRecord {
    pub name: StringId,
    pub name_span: Span,
    pub value: ValueId,
}

#[derive(Clone, Copy)]
pub struct Object<'a>(pub(super) super::Cursor<'a, ValueId>);

impl<'a> Object<'a> {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        let store = self.0.store;
        store.lookup(self.0.id).kind.as_object().unwrap().len()
    }

    pub fn span(&self) -> Span {
        let store = &self.0.store;
        store.lookup(self.0.id).span
    }

    pub fn fields(&self) -> Iter<'a, ObjectField<'a>> {
        let store = self.0.store;
        Iter::new(store.lookup(self.0.id).kind.as_object().unwrap(), store)
    }

    pub fn get(&self, name: &str) -> Option<Value<'a>> {
        Some(self.fields().find(|field| field.name() == name)?.value())
    }
}

impl PartialEq for Object<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self.fields().all(|field| {
                let needle = field.name();
                let Some(b_field) = other
                    .fields()
                    .find(|other_field| other_field.name() == needle)
                else {
                    return false;
                };

                field.value() == b_field.value()
            })
    }
}

impl fmt::Debug for Object<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.fields().map(|field| (field.name(), field.value())))
            .finish()
    }
}

impl<'a> From<ConstObject<'a>> for Object<'a> {
    fn from(value: ConstObject<'a>) -> Self {
        let Cursor { id, store } = value.0;

        let id = id.into();

        Object(Cursor { id, store })
    }
}

impl<'a> IntoIterator for Object<'a> {
    type Item = ObjectField<'a>;

    type IntoIter = Iter<'a, ObjectField<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields()
    }
}

// TODO: Make sure this ObjectField name has no obvious clashes
// and rename if it does
#[derive(Clone, Copy)]
pub struct ObjectField<'a>(super::Cursor<'a, FieldId>);

impl<'a> ObjectField<'a> {
    pub fn name(&self) -> &'a str {
        let store = self.0.store;
        store.lookup(store.lookup(self.0.id).name)
    }

    pub fn name_span(&self) -> Span {
        let store = self.0.store;
        store.lookup(self.0.id).name_span
    }

    pub fn value(&self) -> Value<'a> {
        let store = self.0.store;
        store.read(store.lookup(self.0.id).value)
    }
}

impl<'a> ValueStoreReader<'a> for ObjectField<'a> {
    type Id = FieldId;
}

impl ValueStoreId for FieldId {
    type Reader<'a> = ObjectField<'a>;

    fn read(self, store: &super::ValueStore) -> Self::Reader<'_> {
        ObjectField(super::Cursor { id: self, store })
    }
}
