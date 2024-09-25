use std::iter::Enumerate;

use pretty::{DocAllocator, Pretty};

use crate::printing::pretty::printer::PrettyOptions;

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
pub(super) struct ArgumentSequence<'a> {
    iterator: Enumerate<std::vec::IntoIter<InputValueDefinition<'a>>>,
    options: PrettyOptions,
}

impl<'a> ArgumentSequence<'a> {
    pub fn new(
        iterator: crate::type_system::iter::Iter<'a, InputValueDefinition<'a>>,
        options: PrettyOptions,
    ) -> Self {
        let mut arguments = iterator.collect::<Vec<_>>();
        if options.sort {
            arguments.sort_by_key(|arg| arg.name());
        }

        ArgumentSequence {
            iterator: arguments.into_iter().enumerate(),
            options,
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
            document = document.append(NodeDisplay(item, self.options));
        }
        document
    }
}
