use std::fmt;

use crate::{common::IdRange, AstLookup, Span};

use super::{
    iter::{Iter, ValueStoreReader},
    value::Value,
    ConstFieldId, ConstValueId, ValueStoreId,
};

#[derive(Clone, Copy)]
pub struct ConstObject<'a>(pub(super) super::Cursor<'a, ConstValueId>);

impl<'a> ConstObject<'a> {
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

    pub fn fields(&self) -> Iter<'a, ConstObjectField<'a>> {
        let store = self.0.store;

        let IdRange { start, end } = store.lookup(self.0.id).kind.as_object().unwrap();
        let start = ConstFieldId::new(start.get());
        let end = ConstFieldId::new(end.get());

        Iter::new(IdRange { start, end }, store)
    }
}

impl fmt::Debug for ConstObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.fields().map(|field| (field.name(), field.value())))
            .finish()
    }
}

// TODO: Make sure this ConstObjectField name has no obvious clashes
// and rename if it does
pub struct ConstObjectField<'a>(super::Cursor<'a, ConstFieldId>);

impl<'a> ConstObjectField<'a> {
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

impl<'a> ValueStoreReader<'a> for ConstObjectField<'a> {
    type Id = ConstFieldId;
}

impl ValueStoreId for ConstFieldId {
    type Reader<'a> = ConstObjectField<'a>;

    fn read(self, store: &super::ValueStore) -> Self::Reader<'_> {
        ConstObjectField(super::Cursor { id: self, store })
    }
}
