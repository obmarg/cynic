use std::{fmt, iter::FusedIterator};

use crate::common::{IdOperations, IdRange, IdRangeIter};

use super::{ExecutableDocument, ExecutableId};

pub trait IdReader {
    type Id: ExecutableId;
    type Reader<'a>;

    fn new(id: Self::Id, document: &'_ ExecutableDocument) -> Self::Reader<'_>;
}

/// Iterator for readers in the executable module
///
/// T indicates the type that will be yielded by the Iterator
#[derive(Clone)]
pub struct Iter<'a, T>
where
    T: IdReader,
{
    ids: IdRangeIter<T::Id>,
    document: &'a super::ExecutableDocument,
}

impl<'a, T> Iter<'a, T>
where
    T: IdReader,
{
    pub(crate) fn new(range: IdRange<T::Id>, document: &'a super::ExecutableDocument) -> Self
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

    pub fn with_ids(&self) -> IdIter<'a, T> {
        let Iter { ids, document } = self;

        IdIter {
            ids: ids.clone(),
            document,
        }
    }
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

impl<T> ExactSizeIterator for Iter<'_, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
}

impl<T> FusedIterator for Iter<'_, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
}

impl<T> DoubleEndedIterator for Iter<'_, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
    // Required method
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(T::new(self.ids.next_back()?, self.document))
    }
}

impl<T> fmt::Debug for Iter<'_, T>
where
    T: IdReader + Copy,
    Self: Iterator,
    <Self as Iterator>::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

/// Iterator over an Id & a Reader for that Id
///
/// T indicates the reader type that will be yielded by the Iterator
#[derive(Clone)]
pub struct IdIter<'a, T>
where
    T: IdReader,
{
    ids: IdRangeIter<T::Id>,
    document: &'a super::ExecutableDocument,
}

impl<'a, T> Iterator for IdIter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
    type Item = (T::Id, <T::Id as ExecutableId>::Reader<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.ids.next()?;

        Some((next, self.document.read(next)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.ids.size_hint()
    }
}

impl<T> ExactSizeIterator for IdIter<'_, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
}

impl<T> FusedIterator for IdIter<'_, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
}

impl<T> DoubleEndedIterator for IdIter<'_, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
    // Required method
    fn next_back(&mut self) -> Option<Self::Item> {
        let next = self.ids.next_back()?;

        Some((next, self.document.read(next)))
    }
}

impl<T> fmt::Debug for IdIter<'_, T>
where
    T: IdReader + Copy,
    Self: Iterator,
    <Self as Iterator>::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}
