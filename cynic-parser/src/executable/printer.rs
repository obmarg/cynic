use pretty::{BoxAllocator, DocAllocator, Pretty};

use super::*;

impl super::Ast {
    pub fn to_sdl(&self) -> String {
        let allocator = BoxAllocator;

        // TODO: Possibly want some logic that detects whether we can just print a single
        // selection set or not...

        let builder = allocator
            .intersperse(
                self.definitions().map(|definition| match definition {
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

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<OperationDefinition<'a>> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
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

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<FragmentDefinition<'a>> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
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
            .append(allocator.text(self.0.type_condition()))
            .append(directives_pretty)
            .append(SelectionSetDisplay::new(self.0.selection_set()))
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<VariableDefinition<'a>> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let mut default_pretty = allocator.nil();

        if let Some(default_value) = self.0.default_value() {
            default_pretty = allocator
                .space()
                .append(allocator.text("="))
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
            .append(NodeDisplay(self.0.ty()))
            .append(default_pretty)
            .append(directives_pretty)
    }
}

struct SelectionSetDisplay<'a>(Box<dyn Iterator<Item = Selection<'a>> + 'a>);

impl<'a> SelectionSetDisplay<'a> {
    pub fn new(iter: impl Iterator<Item = Selection<'a>> + 'a) -> Self {
        Self(Box::new(iter))
    }
}

impl<'a> Pretty<'a, BoxAllocator> for SelectionSetDisplay<'a> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let mut selections = self.0.peekable();
        if selections.peek().is_none() {
            return allocator.nil();
        }

        let selections = allocator.concat(selections.map(NodeDisplay)).nest(2);

        allocator
            .space()
            .append(allocator.text("{"))
            .append(allocator.hardline())
            .append(allocator.text("  "))
            .append(selections)
            .append(allocator.text("}"))
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<Selection<'a>> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
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
                        .intersperse(arguments.map(NodeDisplay), comma_or_nil(allocator))
                        .group();

                    arguments_pretty = arguments_pretty
                        .clone()
                        .nest(2)
                        .parens()
                        .flat_alt(arguments_pretty.parens());
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
                    .append(allocator.space())
                    .append(spread.fragment_name())
                    .append(directives_pretty)
            }
        }
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<Type<'a>> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        allocator.text(self.0.to_string())
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<Directive<'a>> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let mut builder = allocator.text(format!("@{}", self.0.name()));

        let mut arguments = self.0.arguments().peekable();

        if arguments.peek().is_some() {
            let arguments = allocator
                .intersperse(arguments.map(NodeDisplay), comma_or_newline(allocator))
                .group()
                .enclose(allocator.softline_(), allocator.softline_());

            builder = builder.append(parens_and_maybe_indent(arguments));
        }

        builder
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<Argument<'a>> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        allocator
            .text(self.0.name().to_string())
            .append(allocator.text(":"))
            .append(allocator.space())
            .append(NodeDisplay(self.0.value()))
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<Value<'a>> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        match self.0 {
            Value::Variable(name) => allocator.text(format!("${name}")),
            Value::Int(value) => allocator.text(format!("{value}")),
            Value::Float(value) => allocator.text(format!("{value}")),
            Value::String(value) => allocator.text(value.to_string()),
            Value::Boolean(value) => allocator.text(format!("{value}")),
            Value::Null => allocator.text("null"),
            Value::Enum(value) => allocator.text(value.to_string()),
            Value::List(items) => brackets_and_maybe_indent(
                allocator
                    .intersperse(
                        items.into_iter().map(NodeDisplay),
                        allocator.text(",").append(allocator.line()),
                    )
                    .group()
                    .enclose(allocator.line_(), allocator.line_()),
            ),
            Value::Object(items) => allocator
                .intersperse(
                    items.into_iter().map(|(name, value)| {
                        allocator
                            .text(name)
                            .append(allocator.text(":"))
                            .append(allocator.space())
                            .append(NodeDisplay(value))
                    }),
                    ",",
                )
                .group()
                .enclose(allocator.space(), allocator.space())
                .braces(),
        }
    }
}

fn comma_or_nil(allocator: &BoxAllocator) -> pretty::DocBuilder<'_, BoxAllocator> {
    allocator
        .nil()
        .flat_alt(allocator.text(",").append(allocator.space()))
}

fn comma_or_newline(allocator: &BoxAllocator) -> pretty::DocBuilder<'_, BoxAllocator> {
    allocator
        .line()
        .flat_alt(allocator.text(",").append(allocator.space()))
}

fn parens_and_maybe_indent(
    thing: pretty::DocBuilder<'_, BoxAllocator>,
) -> pretty::DocBuilder<'_, BoxAllocator> {
    thing.clone().nest(2).parens().flat_alt(thing.parens())
}

fn brackets_and_maybe_indent(
    thing: pretty::DocBuilder<'_, BoxAllocator>,
) -> pretty::DocBuilder<'_, BoxAllocator> {
    thing.clone().nest(2).brackets().flat_alt(thing.brackets())
}
