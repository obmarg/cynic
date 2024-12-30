use std::fmt;

use crate::{common::IdRange, AstLookup, Span};

use super::{const_value::ConstValue, iter::Iter, ConstValueId};

#[derive(Clone, Copy)]
pub struct ConstList<'a>(pub(super) super::Cursor<'a, ConstValueId>);

impl<'a> ConstList<'a> {
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

    pub fn items(&self) -> Iter<'a, ConstValue<'a>> {
        let store = &self.0.store;

        let IdRange { start, end } = store.lookup(self.0.id).kind.as_list().unwrap();
        let start = ConstValueId::new(start.get());
        let end = ConstValueId::new(end.get());

        Iter::new(IdRange { start, end }, store)
    }

    pub fn get(&self, index: usize) -> Option<ConstValue<'a>> {
        self.items().nth(index)
    }
}

impl fmt::Debug for ConstList<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.items()).finish()
    }
}

impl<'a> IntoIterator for ConstList<'a> {
    type Item = ConstValue<'a>;

    type IntoIter = Iter<'a, ConstValue<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.items()
    }
}
