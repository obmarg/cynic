use std::{fmt, iter::FusedIterator};

use crate::common::{IdOperations, IdRange};

use super::ExecutableId;

pub trait IdReader {
    type Id: ExecutableId;
}

/// Iterator for readers in the executable module
///
/// T indicates the type that will be yielded by the Iterator
#[derive(Clone, Copy)]
pub struct Iter<'a, T>
where
    T: IdReader,
{
    range: IdRange<T::Id>,
    document: &'a super::ExecutableDocument,
}

impl<'a, T> Iter<'a, T>
where
    T: IdReader,
{
    pub(crate) fn new(range: IdRange<T::Id>, document: &'a super::ExecutableDocument) -> Self {
        Iter { range, document }
    }

    pub fn ids(&self) -> IdRange<T::Id> {
        self.range
    }

    pub fn with_ids(self) -> IdIter<'a, T> {
        let Iter { range, document } = self;

        IdIter { range, document }
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
    type Item = <T::Id as ExecutableId>::Reader<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.document.read(self.range.next()?))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
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
        Some(self.document.read(self.range.next_back()?))
    }
}

impl<'a, T> fmt::Debug for Iter<'a, T>
where
    T: IdReader + Copy,
    Self: Iterator,
    <Self as Iterator>::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(*self).finish()
    }
}

/// Iterator over an Id & a Reader for that Id
///
/// T indicates the reader type that will be yielded by the Iterator
#[derive(Clone, Copy)]
pub struct IdIter<'a, T>
where
    T: IdReader,
{
    range: IdRange<T::Id>,
    document: &'a super::ExecutableDocument,
}

impl<'a, T> Iterator for IdIter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
    type Item = (T::Id, <T::Id as ExecutableId>::Reader<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.range.next()?;

        Some((next, self.document.read(next)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl<'a, T> ExactSizeIterator for IdIter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
}

impl<'a, T> FusedIterator for IdIter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
}

impl<'a, T> DoubleEndedIterator for IdIter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
    // Required method
    fn next_back(&mut self) -> Option<Self::Item> {
        let next = self.range.next_back()?;

        Some((next, self.document.read(next)))
    }
}

impl<'a, T> fmt::Debug for IdIter<'a, T>
where
    T: IdReader + Copy,
    Self: Iterator,
    <Self as Iterator>::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(*self).finish()
    }
}
