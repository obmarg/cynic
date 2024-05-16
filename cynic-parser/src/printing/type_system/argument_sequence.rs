use std::iter::Enumerate;

use pretty::{DocAllocator, Pretty};

use super::{comma_or_newline, Allocator, InputValueDefinition, NodeDisplay};

/// A sequence of things with docstrings attached.
///
/// This will print each entry with a prefix of:
/// - first entry gets no prefix
/// - when no docstring: just a single hardline prefix
/// - when a docstring: two hardline prefix
///
/// This is used for displaying fields (both input fields & output fields)
/// but arguments should use ArgumentSequence
pub struct ArgumentSequence<'a> {
    iterator: Enumerate<crate::type_system::iter::Iter<'a, InputValueDefinition<'a>>>,
}

impl<'a> ArgumentSequence<'a> {
    pub fn new(iterator: crate::type_system::iter::Iter<'a, InputValueDefinition<'a>>) -> Self {
        ArgumentSequence {
            iterator: iterator.enumerate(),
        }
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for ArgumentSequence<'a> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut document = allocator.nil();
        for (index, item) in self.iterator {
            if index != 0 {
                document = document.append(comma_or_newline(allocator));
                if item.description().is_some() {
                    document = document.append(allocator.hardline());
                }
            }
            document = document.append(NodeDisplay(item));
        }
        document
    }
}
