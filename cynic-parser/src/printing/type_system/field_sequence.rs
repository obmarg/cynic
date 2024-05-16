use std::iter::Enumerate;

use pretty::{DocAllocator, Pretty};

use crate::common::IdOperations;

use super::{
    iter::IdReader, Allocator, FieldDefinition, InputValueDefinition, NodeDisplay, TypeSystemId,
};

/// A sequence of things with docstrings attached.
///
/// This will print each entry with a prefix of:
/// - first entry gets no prefix
/// - when no docstring: just a single hardline prefix
/// - when a docstring: two hardline prefix
///
/// This is used for displaying fields (both input fields & output fields)
/// but arguments should use ArgumentSequence
pub struct FieldSequence<'a, T>
where
    T: IdReader,
{
    iterator: Enumerate<crate::type_system::iter::Iter<'a, T>>,
}

impl<'a, T> FieldSequence<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
    pub fn new(iterator: crate::type_system::iter::Iter<'a, T>) -> Self {
        FieldSequence {
            iterator: iterator.enumerate(),
        }
    }
}

trait DocstringedItem {
    fn has_docstring(&self) -> bool;
}

impl DocstringedItem for FieldDefinition<'_> {
    fn has_docstring(&self) -> bool {
        self.description().is_some()
    }
}

impl DocstringedItem for InputValueDefinition<'_> {
    fn has_docstring(&self) -> bool {
        self.description().is_some()
    }
}

impl<'a, T> Pretty<'a, Allocator<'a>> for FieldSequence<'a, T>
where
    T: DocstringedItem + IdReader,
    T::Id: IdOperations,
    // This is a bit much :/
    <<T as IdReader>::Id as TypeSystemId>::Reader<'a>: DocstringedItem,
    NodeDisplay<<<T as IdReader>::Id as TypeSystemId>::Reader<'a>>: Pretty<'a, Allocator<'a>>,
{
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut document = allocator.nil();
        for (index, item) in self.iterator {
            if index != 0 {
                document = document.append(allocator.hardline());
                if item.has_docstring() {
                    document = document.append(allocator.hardline().nest(-2))
                }
            }
            document = document.append(NodeDisplay(item));
        }
        document
    }
}
