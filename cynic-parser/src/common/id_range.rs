use std::{cmp::Ordering, iter::FusedIterator};

#[derive(Clone, Copy)]
/// A half open range of Ids.
pub struct IdRange<Id> {
    pub(crate) start: Id,
    pub(crate) end: Id,
}

impl<Id> IdRange<Id>
where
    Id: IdOperations,
{
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        IdOperations::distance(self.start, self.end)
    }
}

pub trait IdOperations: Copy {
    fn empty_range() -> IdRange<Self>;
    fn forward(self) -> Option<Self>;
    fn back(self) -> Option<Self>;
    fn cmp(self, other: Self) -> Ordering;
    fn distance(lhs: Self, rhs: Self) -> usize;
}

impl<Id> IdRange<Id> {
    pub(crate) fn new(start: Id, end: Id) -> Self {
        IdRange { start, end }
    }
}

impl<Id> Default for IdRange<Id>
where
    Id: IdOperations,
{
    fn default() -> Self {
        Id::empty_range()
    }
}

impl<Id> IntoIterator for IdRange<Id>
where
    Id: IdOperations,
{
    type Item = Id;

    type IntoIter = IdRangeIter<Id>;

    fn into_iter(self) -> Self::IntoIter {
        IdRangeIter(self)
    }
}

#[derive(Clone)]
pub struct IdRangeIter<Id>(IdRange<Id>);

impl<Id> IdRangeIter<Id> {
    pub fn current_range(&self) -> IdRange<Id>
    where
        Id: Clone,
    {
        self.0.clone()
    }
}

impl<Id> Iterator for IdRangeIter<Id>
where
    Id: IdOperations,
{
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        if IdOperations::cmp(self.0.start, self.0.end).is_eq() {
            return None;
        }
        let current = self.0.start;
        self.0.start = self.0.start.forward()?;
        Some(current)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = IdOperations::distance(self.0.start, self.0.end);
        (size, Some(size))
    }
}

impl<Id> DoubleEndedIterator for IdRangeIter<Id>
where
    Id: IdOperations,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if IdOperations::cmp(self.0.start, self.0.end).is_eq() {
            return None;
        }
        let current = self.0.end.back()?;
        self.0.end = current;
        Some(current)
    }
}

impl<Id> ExactSizeIterator for IdRangeIter<Id> where Id: IdOperations {}
impl<Id> FusedIterator for IdRangeIter<Id> where Id: IdOperations {}

#[cfg(test)]
mod tests {
    use super::{IdOperations, IdRange};

    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    struct TestId(u32);

    #[test]
    fn test_id_range() {
        let default = IdRange::<TestId>::default();
        assert_eq!(default.len(), 0);
        assert_eq!(default.into_iter().len(), 0);
        assert_eq!(default.into_iter().next(), None);

        let range = IdRange::new(TestId(0), TestId(3));
        assert_eq!(
            range.into_iter().collect::<Vec<_>>(),
            vec![TestId(0), TestId(1), TestId(2)]
        );

        assert_eq!(
            range.into_iter().rev().collect::<Vec<_>>(),
            vec![TestId(2), TestId(1), TestId(0)]
        );
    }

    impl IdOperations for TestId {
        fn empty_range() -> IdRange<Self> {
            IdRange::new(TestId(0), TestId(0))
        }

        fn forward(self) -> Option<Self> {
            Some(Self(self.0 + 1))
        }

        fn back(self) -> Option<Self> {
            Some(Self(self.0 - 1))
        }

        fn cmp(self, other: Self) -> std::cmp::Ordering {
            self.0.cmp(&other.0)
        }

        fn distance(lhs: Self, rhs: Self) -> usize {
            rhs.0.saturating_sub(lhs.0) as usize
        }
    }
}
