use std::iter::Enumerate;

use pretty::{DocAllocator, Pretty};

use super::{Allocator, InputValueDefinition, NodeDisplay};

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
                if item.description().is_some() {
                    // We use a hardcoded `\n` for the first newline here because
                    // pretty always adds an indent on line but we want this line blank
                    document = document.append(allocator.text("\n").flat_alt(allocator.text(", ")));
                    document = document.append(allocator.hardline());
                } else {
                    document = document.append(allocator.line().flat_alt(allocator.text(", ")));
                }
            }
            document = document.append(NodeDisplay(item));
        }
        document
    }
}
