use std::fmt;

use crate::{common::IdRange, AstLookup, Span};

use super::{
    iter::{Iter, ValueStoreReader},
    ConstFieldId, ConstValue, ConstValueId, ValueStoreId,
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

    pub fn get(&self, name: &str) -> Option<ConstValue<'a>> {
        Some(self.fields().find(|field| field.name() == name)?.value())
    }
}

impl fmt::Debug for ConstObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.fields().map(|field| (field.name(), field.value())))
            .finish()
    }
}

impl<'a> IntoIterator for ConstObject<'a> {
    type Item = ConstObjectField<'a>;

    type IntoIter = Iter<'a, ConstObjectField<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields()
    }
}

#[derive(Clone, Copy)]
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

    pub fn value(&self) -> ConstValue<'a> {
        let store = self.0.store;
        store.read(ConstValueId::new(store.lookup(self.0.id).value.get()))
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
