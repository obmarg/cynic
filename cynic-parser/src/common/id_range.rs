#[derive(Clone, Copy)]
/// A half open range of Ids.
pub struct IdRange<Id> {
    pub(crate) start: Id,
    pub(crate) end: Id,
}

impl<Id> IdRange<Id> {
    pub(crate) fn new(start: Id, end: Id) -> Self {
        IdRange { start, end }
    }
}
