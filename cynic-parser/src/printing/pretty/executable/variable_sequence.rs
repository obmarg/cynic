use std::iter::Enumerate;

use pretty::{DocAllocator, Pretty};

use crate::{executable::VariableDefinition, printing::pretty::printer::PrettyOptions};

use super::{Allocator, NodeDisplay};

/// A sequence of variables with docstrings attached.
pub(super) struct VariableSequence<'a> {
    iterator: Enumerate<std::vec::IntoIter<VariableDefinition<'a>>>,
    options: PrettyOptions,
}

impl<'a> VariableSequence<'a> {
    pub fn new(
        iterator: crate::executable::iter::Iter<'a, VariableDefinition<'a>>,
        options: PrettyOptions,
    ) -> Self {
        let mut variables = iterator.collect::<Vec<_>>();
        if options.sort {
            variables.sort_by_key(|arg| arg.name());
        }

        VariableSequence {
            iterator: variables.into_iter().enumerate(),
            options,
        }
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for VariableSequence<'a> {
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
