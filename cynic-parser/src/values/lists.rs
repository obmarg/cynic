use std::fmt;

use crate::{AstLookup, Span};

use super::{const_lists::ConstList, ids::ValueId, iter::Iter, value::Value, Cursor};

#[derive(Clone, Copy)]
pub struct List<'a>(pub(super) super::Cursor<'a, ValueId>);

impl<'a> List<'a> {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        let store = &self.0.store;
        store.lookup(self.0.id).kind.as_list().unwrap().len()
    }

    pub fn span(&self) -> Span {
        let store = &self.0.store;
        store.lookup(self.0.id).span
    }

    pub fn items(&self) -> Iter<'a, Value<'a>> {
        let store = &self.0.store;
        Iter::new(store.lookup(self.0.id).kind.as_list().unwrap(), store)
    }

    pub fn get(&self, index: usize) -> Option<Value<'a>> {
        self.items().nth(index)
    }
}

impl PartialEq for List<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.items().zip(other.items()).all(|(lhs, rhs)| lhs == rhs)
    }
}

impl fmt::Debug for List<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.items()).finish()
    }
}

impl<'a> From<ConstList<'a>> for List<'a> {
    fn from(value: ConstList<'a>) -> Self {
        let Cursor { id, store } = value.0;

        let id = id.into();

        List(Cursor { id, store })
    }
}

impl<'a> IntoIterator for List<'a> {
    type Item = Value<'a>;

    type IntoIter = Iter<'a, Value<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.items()
    }
}
