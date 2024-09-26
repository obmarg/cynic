use std::{fmt, iter::FusedIterator};

use crate::common::{IdOperations, IdRange, IdRangeIter};

use super::ValueStoreId;

/// Iterator for readers in the executable module
///
/// T indicates the type that will be yielded by the Iterator
#[derive(Clone)]
pub struct Iter<'a, T>
where
    T: ValueStoreReader<'a>,
{
    ids: IdRangeIter<T::Id>,
    store: &'a super::ValueStore,
}

impl<'a, T> Iter<'a, T>
where
    T: ValueStoreReader<'a>,
{
    pub(crate) fn new(range: IdRange<T::Id>, document: &'a super::ValueStore) -> Self
    where
        T::Id: IdOperations,
    {
        Iter {
            ids: range.into_iter(),
            store: document,
        }
    }

    pub fn ids(&self) -> IdRange<T::Id> {
        self.ids.current_range()
    }
}

pub trait ValueStoreReader<'a> {
    type Id: ValueStoreId<Reader<'a> = Self>;
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: ValueStoreReader<'a>,
    T::Id: IdOperations,
{
    type Item = <T::Id as ValueStoreId>::Reader<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.store.read(self.ids.next()?))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.ids.size_hint()
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T>
where
    T: ValueStoreReader<'a>,
    T::Id: IdOperations,
{
}

impl<'a, T> FusedIterator for Iter<'a, T>
where
    T: ValueStoreReader<'a>,
    T::Id: IdOperations,
{
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: ValueStoreReader<'a>,
    T::Id: IdOperations,
{
    // Required method
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(self.store.read(self.ids.next_back()?))
    }
}

impl<'a, T> fmt::Debug for Iter<'a, T>
where
    T: ValueStoreReader<'a> + Copy,
    Self: Iterator,
    <Self as Iterator>::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}
