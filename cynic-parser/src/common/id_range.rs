use std::{cmp::Ordering, iter::FusedIterator};

#[derive(Clone, Copy)]
/// A half open range of Ids.
pub struct IdRange<Id> {
    pub(crate) start: Id,
    pub(crate) end: Id,
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

impl<Id> Iterator for IdRange<Id>
where
    Id: IdOperations,
{
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        if IdOperations::cmp(self.start, self.end).is_eq() {
            return None;
        }
        let current = self.start;
        self.start = self.start.forward()?;
        Some(current)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = IdOperations::distance(self.start, self.end);
        (size, Some(size))
    }
}

impl<Id> DoubleEndedIterator for IdRange<Id>
where
    Id: IdOperations,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if IdOperations::cmp(self.start, self.end).is_eq() {
            return None;
        }
        let current = self.end.back()?;
        self.end = current;
        Some(current)
    }
}

impl<Id> ExactSizeIterator for IdRange<Id> where Id: IdOperations {}
impl<Id> FusedIterator for IdRange<Id> where Id: IdOperations {}

#[cfg(test)]
mod tests {
    use super::{IdOperations, IdRange};

    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    struct TestId(u32);

    #[test]
    fn test_id_range() {
        let default = IdRange::<TestId>::default();
        assert_eq!(default.len(), 0);
        assert_eq!(default.clone().next(), None);

        let range = IdRange::new(TestId(0), TestId(3));
        assert_eq!(
            range.collect::<Vec<_>>(),
            vec![TestId(0), TestId(1), TestId(2)]
        );

        assert_eq!(
            range.rev().collect::<Vec<_>>(),
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
