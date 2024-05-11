use std::iter::FusedIterator;

use crate::common::{IdOperations, IdRange};

use super::TypeSystemId;

/// Iterator for readers in the type system module
///
/// T indicates the type that will be yielded by the Iterator
#[derive(Clone, Copy)]
pub struct Iter<'a, T>
where
    T: IdReader,
{
    range: IdRange<T::Id>,
    current: T::Id,
    document: &'a super::TypeSystemDocument,
}

impl<'a, T> Iter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
    pub(crate) fn new(range: IdRange<T::Id>, document: &'a super::TypeSystemDocument) -> Self {
        Iter {
            current: range.start,
            range,
            document,
        }
    }
}

pub trait IdReader {
    type Id: TypeSystemId;
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
    type Item = <T::Id as TypeSystemId>::Reader<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.range.next(self.current)?;
        self.current = next;

        Some(self.document.read(next))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = IdOperations::distance(self.current, self.range.end);
        (remaining, Some(remaining))
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
        let next = self.range.previous(self.current)?;
        self.current = next;

        Some(self.document.read(next))
    }
}
