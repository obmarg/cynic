use pretty::{DocAllocator, Pretty};

use crate::common::OperationType;

use crate::executable::*;

use super::escape_string;

type Allocator<'a> = pretty::Arena<'a>;

impl crate::ExecutableDocument {
    pub fn to_executable_string(&self) -> String {
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

        let builder = allocator
            .intersperse(
                self.definitions().map(|definition| match definition {
                    ExecutableDefinition::Operation(reader) if use_short_form => {
                        SelectionSetDisplay::new(reader.selection_set())
                            .without_leading_space()
                            .pretty(&allocator)
                    }
                    ExecutableDefinition::Operation(reader) => {
                        NodeDisplay(reader).pretty(&allocator)
                    }
                    ExecutableDefinition::Fragment(reader) => {
                        NodeDisplay(reader).pretty(&allocator)
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

pub struct NodeDisplay<T>(T);

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<OperationDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut builder = allocator.text(self.0.operation_type().as_str());

        if let Some(name) = self.0.name() {
            builder = builder
                .append(allocator.space())
                .append(allocator.text(name));
        }

        let mut variables_pretty = allocator.nil();
        let mut variables = self.0.variable_definitions().peekable();
        if variables.peek().is_some() {
            variables_pretty = allocator
                .intersperse(variables.map(NodeDisplay), comma_or_nil(allocator))
                .group();

            variables_pretty = variables_pretty
                .clone()
                .nest(2)
                .parens()
                .flat_alt(variables_pretty.parens());
        }

        let mut directives = self.0.directives().peekable();
        let mut directives_pretty = allocator.nil();
        if directives.peek().is_some() {
            directives_pretty = allocator
                .space()
                .append(allocator.intersperse(directives.map(NodeDisplay), allocator.space()));
        }

        builder
            .append(variables_pretty)
            .append(directives_pretty)
            .append(SelectionSetDisplay::new(self.0.selection_set()))
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<FragmentDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut directives = self.0.directives().peekable();
        let mut directives_pretty = allocator.nil();
        if directives.peek().is_some() {
            directives_pretty = allocator
                .space()
                .append(allocator.intersperse(directives.map(NodeDisplay), allocator.space()));
        }

        allocator
            .text("fragment")
            .append(allocator.space())
            .append(allocator.text(self.0.name()))
            .append(allocator.space())
            .append(allocator.text("on"))
            .append(allocator.space())
            .append(allocator.text(self.0.type_condition()))
            .append(directives_pretty)
            .append(SelectionSetDisplay::new(self.0.selection_set()))
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
                .append(NodeDisplay(default_value))
        }

        let mut directives = self.0.directives().peekable();
        let mut directives_pretty = allocator.nil();
        if directives.peek().is_some() {
            directives_pretty = allocator
                .space()
                .append(allocator.intersperse(directives.map(NodeDisplay), allocator.space()));
        }

        allocator
            .text("$")
            .append(allocator.text(self.0.name()))
            .append(allocator.text(":"))
            .append(allocator.space())
            .append(NodeDisplay(self.0.ty()))
            .append(default_pretty)
            .append(directives_pretty)
    }
}

struct SelectionSetDisplay<'a> {
    selections: Box<dyn Iterator<Item = Selection<'a>> + 'a>,
    leading_space: bool,
}

impl<'a> SelectionSetDisplay<'a> {
    pub fn new(iter: impl ExactSizeIterator<Item = Selection<'a>> + 'a) -> Self {
        Self {
            selections: Box::new(iter),
            leading_space: true,
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
            .intersperse(selections.map(NodeDisplay), allocator.hardline())
            .align();

        let mut builder = allocator.nil();

        if self.leading_space {
            builder = allocator.space();
        }

        builder
            .append(allocator.text("{"))
            .append(allocator.hardline())
            .append(allocator.text("  "))
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
                    arguments_pretty = allocator
                        .intersperse(arguments.map(NodeDisplay), comma_or_newline(allocator))
                        .group();

                    arguments_pretty = parens_and_maybe_indent(allocator, arguments_pretty);
                }

                let mut directives = field.directives().peekable();
                let mut directives_pretty = allocator.nil();
                if directives.peek().is_some() {
                    directives_pretty = allocator.space().append(
                        allocator.intersperse(directives.map(NodeDisplay), allocator.space()),
                    );
                }

                alias_pretty
                    .append(allocator.text(field.name()))
                    .append(arguments_pretty)
                    .append(directives_pretty)
                    .append(SelectionSetDisplay::new(field.selection_set()))
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
                        allocator.intersperse(directives.map(NodeDisplay), allocator.space()),
                    );
                }

                allocator
                    .text("...")
                    .append(type_condition_pretty)
                    .append(directives_pretty)
                    .append(SelectionSetDisplay::new(fragment.selection_set()))
            }
            Selection::FragmentSpread(spread) => {
                let mut directives = spread.directives().peekable();
                let mut directives_pretty = allocator.nil();
                if directives.peek().is_some() {
                    directives_pretty = allocator.space().append(
                        allocator.intersperse(directives.map(NodeDisplay), allocator.space()),
                    );
                }

                allocator
                    .text("...")
                    .append(spread.fragment_name())
                    .append(directives_pretty)
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
                .intersperse(arguments.map(NodeDisplay), comma_or_newline(allocator))
                .group();

            builder = builder.append(parens_and_maybe_indent(allocator, arguments));
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
            .append(NodeDisplay(self.0.value()))
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<Value<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        match self.0 {
            Value::Variable(name) => allocator.text(format!("${name}")),
            Value::Int(value) => allocator.text(format!("{value}")),
            Value::Float(value) => allocator.text(format!("{value}")),
            Value::String(value) => allocator.text(escape_string(value)).double_quotes(),
            Value::Boolean(value) => allocator.text(format!("{value}")),
            Value::Null => allocator.text("null"),
            Value::Enum(value) => allocator.text(value),
            Value::List(items) if items.is_empty() => allocator.nil().brackets(),
            Value::List(items) => brackets_and_maybe_indent(
                allocator
                    .intersperse(
                        items.into_iter().map(NodeDisplay),
                        allocator.text(",").append(allocator.line()),
                    )
                    .group()
                    .enclose(allocator.line_(), allocator.line_()),
            ),
            Value::Object(items) if items.is_empty() => allocator.nil().braces(),
            Value::Object(items) => allocator
                .intersperse(
                    items.into_iter().map(|(name, value)| {
                        allocator
                            .text(name)
                            .append(allocator.text(":"))
                            .append(allocator.space())
                            .append(NodeDisplay(value))
                    }),
                    allocator.text(",").append(allocator.space()),
                )
                .group()
                .enclose(allocator.softline(), allocator.softline())
                .braces(),
        }
    }
}

fn comma_or_nil<'a>(allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>> {
    allocator
        .nil()
        .flat_alt(allocator.text(",").append(allocator.space()))
}

fn comma_or_newline<'a>(allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'_, Allocator<'a>> {
    allocator
        .line()
        .flat_alt(allocator.text(",").append(allocator.space()))
}

fn parens_and_maybe_indent<'a>(
    allocator: &'a Allocator<'a>,
    thing: pretty::DocBuilder<'a, Allocator<'a>>,
) -> pretty::DocBuilder<'a, Allocator<'a>> {
    thing
        .clone()
        .enclose(allocator.softline_(), allocator.softline_())
        .nest(2)
        .parens()
        .flat_alt(thing.parens())
}

fn brackets_and_maybe_indent<'a>(
    thing: pretty::DocBuilder<'a, Allocator<'a>>,
) -> pretty::DocBuilder<'a, Allocator<'a>> {
    thing.clone().nest(2).brackets().flat_alt(thing.brackets())
}
