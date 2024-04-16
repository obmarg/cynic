use crate::common::IdRange;

use super::ExecutableId;

/// Iterator for readers in the executable module
///
/// T indicates the type that will be yielded by the Iterator
struct Iter<'a, T>
where
    T: IdReader,
{
    range: IdRange<T::Id>,
    next: Option<T::Id>,
    document: &'a super::ExecutableDocument,
}

pub trait IdReader {
    type Id: ExecutableId;
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: IdReader,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.next()? {
            let next = current.
        }
    }
}
