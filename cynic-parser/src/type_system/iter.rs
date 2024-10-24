use std::{fmt, iter::FusedIterator};

use crate::common::{IdOperations, IdRange, IdRangeIter};

use super::{TypeSystemDocument, TypeSystemId};

/// Iterator for readers in the executable module
///
/// T indicates the type that will be yielded by the Iterator
#[derive(Clone)]
pub struct Iter<'a, T>
where
    T: IdReader,
{
    ids: IdRangeIter<T::Id>,
    document: &'a super::TypeSystemDocument,
}

impl<'a, T> Iter<'a, T>
where
    T: IdReader,
{
    pub(crate) fn new(range: IdRange<T::Id>, document: &'a super::TypeSystemDocument) -> Self
    where
        T::Id: IdOperations,
    {
        Iter {
            ids: range.into_iter(),
            document,
        }
    }

    pub fn ids(&self) -> IdRange<T::Id> {
        self.ids.current_range()
    }
}

pub trait IdReader {
    type Id: TypeSystemId;
    type Reader<'a>;

    fn new(id: Self::Id, document: &'_ TypeSystemDocument) -> Self::Reader<'_>;
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
    type Item = T::Reader<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(T::new(self.ids.next()?, self.document))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.ids.size_hint()
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
}

impl<'a, T> FusedIterator for Iter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
    // Required method
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(T::new(self.ids.next_back()?, self.document))
    }
}

impl<'a, T> fmt::Debug for Iter<'a, T>
where
    T: IdReader + Copy,
    Self: Iterator,
    <Self as Iterator>::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}
