use std::iter::Enumerate;

use pretty::{DocAllocator, Pretty};

use crate::{common::IdOperations, printing::pretty::printer::PrettyOptions};

use super::{
    iter::IdReader, Allocator, EnumValueDefinition, FieldDefinition, InputValueDefinition,
    NodeDisplay, TypeSystemId,
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
pub(super) struct FieldSequence<'a, T>
where
    T: IdReader + Field<'a>,
{
    iterator: Enumerate<std::vec::IntoIter<<<T as IdReader>::Id as TypeSystemId>::Reader<'a>>>,
    options: PrettyOptions,
}

impl<'a, T> FieldSequence<'a, T>
where
    T: IdReader + Field<'a>,
    T::Id: IdOperations,
    <<T as IdReader>::Id as TypeSystemId>::Reader<'a>: Field<'a>,
{
    pub fn new(iterator: crate::type_system::iter::Iter<'a, T>, options: PrettyOptions) -> Self {
        let mut fields = iterator.collect::<Vec<_>>();
        if options.sort {
            fields.sort_by_key(|arg| arg.name());
        }

        FieldSequence {
            iterator: fields.into_iter().enumerate(),
            options,
        }
    }
}

pub trait Field<'a> {
    fn name(&self) -> &'a str;
    fn has_docstring(&self) -> bool;
}

impl<'a> Field<'a> for FieldDefinition<'a> {
    fn name(&self) -> &'a str {
        FieldDefinition::name(self)
    }

    fn has_docstring(&self) -> bool {
        self.description().is_some()
    }
}

impl<'a> Field<'a> for InputValueDefinition<'a> {
    fn name(&self) -> &'a str {
        InputValueDefinition::name(self)
    }

    fn has_docstring(&self) -> bool {
        self.description().is_some()
    }
}

impl<'a> Field<'a> for EnumValueDefinition<'a> {
    fn name(&self) -> &'a str {
        EnumValueDefinition::value(self)
    }

    fn has_docstring(&self) -> bool {
        self.description().is_some()
    }
}

impl<'a, T> Pretty<'a, Allocator<'a>> for FieldSequence<'a, T>
where
    T: Field<'a> + IdReader,
    T::Id: IdOperations,
    // This is a bit much :/
    <<T as IdReader>::Id as TypeSystemId>::Reader<'a>: Field<'a>,
    NodeDisplay<<<T as IdReader>::Id as TypeSystemId>::Reader<'a>>: Pretty<'a, Allocator<'a>>,
{
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut document = allocator.nil();
        for (index, item) in self.iterator {
            if index != 0 {
                if item.has_docstring() {
                    // We use a hardcoded `\n` for the first newline here because
                    // pretty always adds an indent but we want this line blank
                    document = document
                        .append(allocator.text("\n"))
                        .append(allocator.hardline());
                } else {
                    document = document.append(allocator.hardline());
                }
            }
            document = document.append(NodeDisplay(item, self.options));
        }
        document
    }
}
