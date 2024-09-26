use std::fmt;

use crate::{AstLookup, Span};

use super::{
    ids::{FieldId, StringId, ValueId},
    iter::{Iter, ValueStoreReader},
    value::Value,
    ValueStoreId,
};

pub struct FieldRecord {
    pub name: StringId,
    pub name_span: Span,
    pub value: ValueId,
}

pub struct ObjectValue<'a>(pub(super) super::Cursor<'a, ValueId>);

impl<'a> ObjectValue<'a> {
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
}

impl fmt::Debug for ObjectValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.fields().map(|field| (field.name(), field.value())))
            .finish()
    }
}

// TODO: Make sure this ObjectField name has no obvious clashes
// and rename if it does
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
