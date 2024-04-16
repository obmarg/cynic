use std::cmp::Ordering;

#[derive(Clone, Copy)]
/// A half open range of Ids.
pub struct IdRange<Id> {
    pub(crate) start: Id,
    pub(crate) end: Id,
}

pub trait IdOperations: Copy {
    fn forward(self) -> Option<Self>;
    fn back(self) -> Option<Self>;
    fn cmp(self, other: Self) -> Ordering;
    fn distance(lhs: Self, rhs: Self) -> usize;
}

impl<Id> IdRange<Id> {
    pub(crate) fn new(start: Id, end: Id) -> Self {
        IdRange { start, end }
    }

    pub(crate) fn next(&self, current: Id) -> Option<Id>
    where
        Id: IdOperations,
    {
        let next = current.forward()?;
        matches!(next.cmp(self.end), Ordering::Less).then_some(next)
    }

    pub(crate) fn previous(&self, current: Id) -> Option<Id>
    where
        Id: IdOperations,
    {
        let next = current.back()?;
        matches!(next.cmp(self.start), Ordering::Equal | Ordering::Greater).then_some(next)
    }

    pub(crate) fn contains(&self, id: Id) -> bool
    where
        Id: IdOperations,
    {
        matches!(id.cmp(self.start), Ordering::Equal | Ordering::Greater)
            && matches!(id.cmp(self.end), Ordering::Less)
    }
}
