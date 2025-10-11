mod variable_sequence;

use pretty::{DocAllocator, Pretty};

use crate::common::OperationType;

use crate::executable::*;
use crate::printing::pretty::executable::variable_sequence::VariableSequence;

use super::printer::PrettyOptions;
use super::{Allocator, NodeDisplay};

impl crate::ExecutableDocument {
    pub fn to_string_pretty(&self) -> String {
        self.pretty_printer().to_string()
    }

    // TODO: Make this public at some point?
    fn pretty_printer(&self) -> super::PrettyPrinter<'_> {
        super::PrettyPrinter::new_executable(self)
    }

    pub(super) fn pretty_print(&self, _options: &PrettyOptions) -> String {
        let allocator = pretty::Arena::new();

        let use_short_form = {
            let mut operation_iter = self.operations();
            let first_op = operation_iter.next();
            match (first_op, operation_iter.next()) {
                (Some(first_op), None) => {
                    first_op.name().is_none()
                        && first_op.operation_type() == OperationType::Query
                        && first_op.directives().len() == 0
                        && first_op.variable_definitions().len() == 0
                }
                _ => false,
            }
        };

        let options = PrettyOptions::default();

        let builder = allocator
            .intersperse(
                self.definitions().map(|definition| match definition {
                    ExecutableDefinition::Operation(reader) if use_short_form => {
                        SelectionSetDisplay::new(reader.selection_set(), options)
                            .without_leading_space()
                            .pretty(&allocator)
                    }
                    ExecutableDefinition::Operation(reader) => {
                        NodeDisplay(reader, options).pretty(&allocator)
                    }
                    ExecutableDefinition::Fragment(reader) => {
                        NodeDisplay(reader, options).pretty(&allocator)
                    }
                }),
                allocator.concat([allocator.hardline(), allocator.hardline()]),
            )
            .pretty(&allocator);

        #[allow(clippy::needless_borrow)] // This doesn't work without the borrow :|
        {
            format!("{}\n", (&*builder).pretty(80))
        }
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<OperationDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(self.with_node(description))
                .append(allocator.hardline());
        }

        builder = builder.append(allocator.text(self.0.operation_type().as_str()));

        if let Some(name) = self.0.name() {
            builder = builder
                .append(allocator.space())
                .append(allocator.text(name));
        }

        let mut variables_pretty = allocator.nil();
        let variables = self.0.variable_definitions();
        if variables.len() != 0 {
            variables_pretty = allocator
                .line_()
                .append(VariableSequence::new(variables, self.1))
                .nest(2)
                .append(allocator.line_())
                .parens()
                .group()
        }

        let mut directives = self.0.directives().peekable();
        let mut directives_pretty = allocator.nil();
        if directives.peek().is_some() {
            directives_pretty = allocator
                .space()
                .append(allocator.intersperse(directives.map(self.mapper()), allocator.space()));
        }

        builder
            .append(variables_pretty)
            .append(directives_pretty)
            .append(SelectionSetDisplay::new(self.0.selection_set(), self.1))
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<FragmentDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut directives = self.0.directives().peekable();
        let mut directives_pretty = allocator.nil();
        if directives.peek().is_some() {
            directives_pretty = allocator
                .space()
                .append(allocator.intersperse(directives.map(self.mapper()), allocator.space()));
        }

        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(self.with_node(description))
                .append(allocator.hardline());
        }

        builder
            .append(allocator.text("fragment"))
            .append(allocator.space())
            .append(allocator.text(self.0.name()))
            .append(allocator.space())
            .append(allocator.text("on"))
            .append(allocator.space())
            .append(allocator.text(self.0.type_condition()))
            .append(directives_pretty)
            .append(SelectionSetDisplay::new(self.0.selection_set(), self.1))
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<VariableDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut default_pretty = allocator.nil();

        if let Some(default_value) = self.0.default_value() {
            default_pretty = allocator
                .space()
                .append(allocator.text("="))
                .append(allocator.space())
                .append(self.with_node(default_value))
        }

        let mut directives = self.0.directives().peekable();
        let mut directives_pretty = allocator.nil();
        if directives.peek().is_some() {
            directives_pretty = allocator
                .space()
                .append(allocator.intersperse(directives.map(self.mapper()), allocator.space()));
        }

        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(self.with_node(description))
                .append(allocator.hardline());
        }

        builder
            .append(allocator.text("$"))
            .append(allocator.text(self.0.name()))
            .append(allocator.text(":"))
            .append(allocator.space())
            .append(self.with_node(self.0.ty()))
            .append(default_pretty)
            .append(directives_pretty)
    }
}

struct SelectionSetDisplay<'a> {
    selections: Box<dyn Iterator<Item = Selection<'a>> + 'a>,
    leading_space: bool,
    options: PrettyOptions,
}

impl<'a> SelectionSetDisplay<'a> {
    pub fn new(
        iter: impl ExactSizeIterator<Item = Selection<'a>> + 'a,
        options: PrettyOptions,
    ) -> Self {
        Self {
            selections: Box::new(iter),
            leading_space: true,
            options,
        }
    }

    pub fn without_leading_space(self) -> Self {
        Self {
            leading_space: false,
            ..self
        }
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for SelectionSetDisplay<'a> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut selections = self.selections.peekable();
        if selections.peek().is_none() {
            return allocator.nil();
        }

        let selections = allocator
            .hardline()
            .append(allocator.intersperse(
                selections.map(|selection| NodeDisplay(selection, self.options)),
                allocator.hardline(),
            ))
            .nest(2);

        let mut builder = allocator.nil();

        if self.leading_space {
            builder = allocator.space();
        }

        builder
            .append(allocator.text("{"))
            .append(selections)
            .append(allocator.hardline())
            .append(allocator.text("}"))
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<Selection<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        match self.0 {
            Selection::Field(field) => {
                let mut alias_pretty = allocator.nil();
                if let Some(alias) = field.alias() {
                    alias_pretty = allocator
                        .text(alias)
                        .append(allocator.text(":"))
                        .append(allocator.space());
                }

                let mut arguments = field.arguments().peekable();

                let mut arguments_pretty = allocator.nil();
                if arguments.peek().is_some() {
                    arguments_pretty =
                        allocator
                            .line_()
                            .append(allocator.intersperse(
                                arguments.map(self.mapper()),
                                comma_or_newline(allocator),
                            ))
                            .nest(2)
                            .append(allocator.line_())
                            .parens()
                            .group();
                }

                let mut directives = field.directives().peekable();
                let mut directives_pretty = allocator.nil();
                if directives.peek().is_some() {
                    directives_pretty = allocator.space().append(
                        allocator.intersperse(directives.map(self.mapper()), allocator.space()),
                    );
                }

                alias_pretty
                    .append(allocator.text(field.name()))
                    .append(arguments_pretty)
                    .append(directives_pretty)
                    .append(SelectionSetDisplay::new(field.selection_set(), self.1))
                    .group()
            }
            Selection::InlineFragment(fragment) => {
                let mut type_condition_pretty = allocator.nil();
                if let Some(type_condition) = fragment.type_condition() {
                    type_condition_pretty = allocator
                        .space()
                        .append(allocator.text("on"))
                        .append(allocator.space())
                        .append(allocator.text(type_condition));
                }

                let mut directives = fragment.directives().peekable();
                let mut directives_pretty = allocator.nil();
                if directives.peek().is_some() {
                    directives_pretty = allocator.space().append(
                        allocator.intersperse(directives.map(self.mapper()), allocator.space()),
                    );
                }

                allocator
                    .text("...")
                    .append(type_condition_pretty)
                    .append(directives_pretty)
                    .append(SelectionSetDisplay::new(fragment.selection_set(), self.1))
                    .group()
            }
            Selection::FragmentSpread(spread) => {
                let mut directives = spread.directives().peekable();
                let mut directives_pretty = allocator.nil();
                if directives.peek().is_some() {
                    directives_pretty = allocator.space().append(
                        allocator.intersperse(directives.map(self.mapper()), allocator.space()),
                    );
                }

                allocator
                    .text("...")
                    .append(spread.fragment_name())
                    .append(directives_pretty)
                    .group()
            }
        }
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<Type<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        allocator.text(self.0.to_string())
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<Directive<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut builder = allocator.text("@").append(self.0.name());

        let mut arguments = self.0.arguments().peekable();

        if arguments.peek().is_some() {
            let arguments = allocator
                .intersperse(arguments.map(self.mapper()), comma_or_newline(allocator))
                .nest(2)
                .parens()
                .group();

            builder = builder.append(arguments);
        }

        builder
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<Argument<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        allocator
            .text(self.0.name())
            .append(allocator.text(":"))
            .append(allocator.space())
            .append(self.with_node(self.0.value()))
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<Description<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        NodeDisplay(self.0.literal(), self.1).pretty(allocator)
    }
}

fn comma_or_nil<'a>(allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>> {
    allocator
        .nil()
        .flat_alt(allocator.text(",").append(allocator.space()))
}

fn comma_or_newline<'a>(allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>> {
    allocator
        .line()
        .flat_alt(allocator.text(",").append(allocator.space()))
}
