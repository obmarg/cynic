mod argument_sequence;
mod field_sequence;

use pretty::{DocAllocator, Pretty};

use crate::type_system::*;

use self::{argument_sequence::ArgumentSequence, field_sequence::FieldSequence};

use super::{Allocator, NodeDisplay, printer::PrettyOptions, sorting::sort_key_for};

impl crate::TypeSystemDocument {
    pub fn to_sdl_pretty(&self) -> String {
        self.pretty_printer().to_string()
    }

    pub fn pretty_printer(&self) -> super::PrettyPrinter<'_> {
        super::PrettyPrinter::new_type_system(self)
    }

    pub(super) fn pretty_print(&self, options: &PrettyOptions) -> String {
        let allocator = pretty::Arena::new();

        let definitions: Box<dyn Iterator<Item = Definition<'_>>> = match options.sort {
            false => Box::new(self.definitions()),
            true => {
                let mut definitions = self.definitions().collect::<Vec<_>>();
                definitions.sort_by_key(sort_key_for);
                Box::new(definitions.into_iter())
            }
        };

        let builder = allocator
            .intersperse(
                definitions.map(|definition| match definition {
                    Definition::Schema(reader) => NodeDisplay(reader, *options).pretty(&allocator),
                    Definition::SchemaExtension(reader) => allocator
                        .text("extend")
                        .append(allocator.space())
                        .append(NodeDisplay(reader, *options).pretty(&allocator)),
                    Definition::Type(TypeDefinition::Scalar(reader)) => {
                        NodeDisplay(reader, *options).pretty(&allocator)
                    }
                    Definition::Type(TypeDefinition::Object(reader)) => {
                        NodeDisplay(reader, *options).pretty(&allocator)
                    }
                    Definition::Type(TypeDefinition::Interface(reader)) => {
                        NodeDisplay(reader, *options).pretty(&allocator)
                    }
                    Definition::Type(TypeDefinition::Union(reader)) => {
                        NodeDisplay(reader, *options).pretty(&allocator)
                    }
                    Definition::Type(TypeDefinition::Enum(reader)) => {
                        NodeDisplay(reader, *options).pretty(&allocator)
                    }
                    Definition::Type(TypeDefinition::InputObject(reader)) => {
                        NodeDisplay(reader, *options).pretty(&allocator)
                    }
                    Definition::TypeExtension(TypeDefinition::Scalar(reader)) => allocator
                        .text("extend")
                        .append(allocator.space())
                        .append(NodeDisplay(reader, *options).pretty(&allocator)),
                    Definition::TypeExtension(TypeDefinition::Object(reader)) => allocator
                        .text("extend")
                        .append(allocator.space())
                        .append(NodeDisplay(reader, *options).pretty(&allocator)),
                    Definition::TypeExtension(TypeDefinition::Interface(reader)) => allocator
                        .text("extend")
                        .append(allocator.space())
                        .append(NodeDisplay(reader, *options).pretty(&allocator)),
                    Definition::TypeExtension(TypeDefinition::Union(reader)) => allocator
                        .text("extend")
                        .append(allocator.space())
                        .append(NodeDisplay(reader, *options).pretty(&allocator)),
                    Definition::TypeExtension(TypeDefinition::Enum(reader)) => allocator
                        .text("extend")
                        .append(allocator.space())
                        .append(NodeDisplay(reader, *options).pretty(&allocator)),
                    Definition::TypeExtension(TypeDefinition::InputObject(reader)) => allocator
                        .text("extend")
                        .append(allocator.space())
                        .append(NodeDisplay(reader, *options).pretty(&allocator)),
                    Definition::Directive(reader) => {
                        NodeDisplay(reader, *options).pretty(&allocator)
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

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<SchemaDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(self.with_node(description))
                .append(allocator.hardline());
        }

        builder = builder.append(allocator.text("schema"));

        let mut directives = self.0.directives().peekable();
        let mut directives_pretty = allocator.nil();
        if directives.peek().is_some() {
            directives_pretty = allocator
                .space()
                .append(allocator.intersperse(directives.map(self.mapper()), allocator.softline()));
        }

        builder = builder.append(directives_pretty);

        let mut roots = self.0.root_operations().peekable();
        if roots.peek().is_some() {
            builder = builder
                .append(allocator.space())
                .append(allocator.text("{"))
                .append(allocator.hardline())
                .append(
                    allocator
                        .intersperse(
                            roots.map(|operation| {
                                allocator
                                    .text(operation.operation_type().as_str())
                                    .append(": ")
                                    .append(operation.named_type())
                            }),
                            allocator.hardline(),
                        )
                        .indent(2),
                )
                .append(allocator.hardline())
                .append(allocator.text("}"))
        }

        builder
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<ScalarDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(self.with_node(description))
                .append(allocator.hardline());
        }

        let mut directives = self.0.directives().peekable();
        let mut directives_pretty = allocator.nil();
        if directives.peek().is_some() {
            directives_pretty = allocator
                .space()
                .append(allocator.intersperse(directives.map(self.mapper()), allocator.softline()));
        }

        builder
            .append(allocator.text(format!("scalar {}", self.0.name())))
            .append(directives_pretty)
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<ObjectDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(self.with_node(description))
                .append(allocator.hardline());
        }

        builder = builder.append(allocator.text(format!("type {}", self.0.name())));

        let interfaces = self.0.implements_interfaces().collect::<Vec<_>>();

        if !interfaces.is_empty() {
            builder = builder
                .append(allocator.space())
                .append(allocator.text("implements"))
                .append(allocator.space())
                .append(allocator.intersperse(interfaces, " & "))
        }

        let mut directives = self.0.directives().peekable();
        if directives.peek().is_some() {
            builder = builder.append(
                allocator
                    .line()
                    .append(allocator.intersperse(directives.map(self.mapper()), allocator.line()))
                    .nest(2)
                    .group(),
            );
        }

        let fields = self.0.fields();
        if fields.len() != 0 {
            builder = builder
                .append(allocator.space())
                .append(allocator.text("{"))
                .append(
                    allocator
                        .hardline()
                        .append(FieldSequence::new(fields, self.1))
                        .nest(2),
                )
                .append(allocator.hardline())
                .append(allocator.text("}"));
        }

        builder
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<FieldDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let description = self
            .0
            .description()
            .map(|description| {
                allocator
                    .nil()
                    .append(self.with_node(description))
                    .append(allocator.hardline())
            })
            .unwrap_or(allocator.softline_());

        let mut arguments_pretty = allocator.nil();

        let arguments = self.0.arguments();

        if arguments.len() != 0 {
            arguments_pretty = allocator
                .line_()
                .append(ArgumentSequence::new(arguments, self.1))
                .nest(2)
                .append(allocator.line_())
                .parens()
                .group();
        }

        let mut directives = self.0.directives().peekable();
        let mut directives_pretty = allocator.nil();
        if directives.peek().is_some() {
            directives_pretty = allocator
                .line()
                .append(allocator.intersperse(directives.map(self.mapper()), allocator.line()))
                .nest(2)
                .group();
        }

        description
            .append(allocator.text(self.0.name()))
            .append(arguments_pretty)
            .append(allocator.text(":"))
            .append(allocator.space())
            .append(self.with_node(self.0.ty()))
            .append(directives_pretty)
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<InterfaceDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(self.with_node(description))
                .append(allocator.hardline());
        }

        builder = builder.append(allocator.text(format!("interface {}", self.0.name())));

        let interfaces = self.0.implements_interfaces().collect::<Vec<_>>();
        if !interfaces.is_empty() {
            builder = builder
                .append(allocator.space())
                .append(allocator.text("implements"))
                .append(allocator.space())
                .append(allocator.intersperse(interfaces, " & "))
        }

        let mut directives = self.0.directives().peekable();
        if directives.peek().is_some() {
            builder = builder.append(
                allocator
                    .line()
                    .append(allocator.intersperse(directives.map(self.mapper()), allocator.line()))
                    .nest(2)
                    .group(),
            );
        }

        let fields = self.0.fields();
        if fields.len() != 0 {
            builder = builder
                .append(allocator.space())
                .append(allocator.text("{"))
                .append(
                    allocator
                        .hardline()
                        .append(FieldSequence::new(fields, self.1))
                        .nest(2),
                )
                .append(allocator.hardline())
                .append(allocator.text("}"));
        }

        builder
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<UnionDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut builder = allocator.nil();

        builder = builder.append(allocator.text(format!("union {}", self.0.name())));

        let mut directives = self.0.directives().peekable();
        if directives.peek().is_some() {
            builder = builder.append(
                allocator
                    .line()
                    .append(allocator.intersperse(directives.map(self.mapper()), allocator.line()))
                    .nest(2)
                    .group(),
            );
        }

        let mut members = self.0.members().map(|member| member.name()).peekable();

        if members.peek().is_some() {
            let members = allocator
                .line()
                .append(
                    allocator
                        .space()
                        .append(allocator.space())
                        .flat_alt(allocator.nil()),
                )
                .append(
                    allocator.intersperse(members, allocator.line().append(allocator.text("| "))),
                )
                .nest(2);

            builder = builder
                .append(allocator.space())
                .append(allocator.text("="))
                .append(members)
        }

        builder = builder.group();

        if let Some(description) = self.0.description() {
            builder = self
                .with_node(description)
                .pretty(allocator)
                .append(allocator.hardline())
                .append(builder)
                .group();
        }

        builder
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<EnumDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(self.with_node(description))
                .append(allocator.hardline());
        }

        builder = builder.append(allocator.text(format!("enum {}", self.0.name())));

        let mut directives = self.0.directives().peekable();
        if directives.peek().is_some() {
            builder = builder.append(
                allocator
                    .line()
                    .append(allocator.intersperse(directives.map(self.mapper()), allocator.line()))
                    .nest(2)
                    .group(),
            );
        }

        let values = self.0.values();

        if values.len() != 0 {
            builder = builder
                .append(allocator.space())
                .append(allocator.text("{"))
                .append(
                    allocator
                        .hardline()
                        .append(FieldSequence::new(values, self.1))
                        .nest(2),
                )
                .append(allocator.hardline())
                .append(allocator.text("}"));
        }

        builder
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<EnumValueDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let description = self
            .0
            .description()
            .map(|description| {
                allocator
                    .nil()
                    .append(self.with_node(description))
                    .append(allocator.hardline())
            })
            .unwrap_or(allocator.softline_());

        let mut builder = description.append(allocator.text(self.0.value()));

        let mut directives = self.0.directives().peekable();
        if directives.peek().is_some() {
            let directives_pretty = allocator
                .line()
                .append(allocator.intersperse(directives.map(self.mapper()), allocator.line()))
                .nest(2)
                .group();

            builder = builder.append(directives_pretty);
        }

        builder
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<InputObjectDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(self.with_node(description))
                .append(allocator.hardline());
        }

        builder = builder.append(allocator.text(format!("input {}", self.0.name())));

        let mut directives = self.0.directives().peekable();
        if directives.peek().is_some() {
            builder = builder.append(
                allocator
                    .line()
                    .append(allocator.intersperse(directives.map(self.mapper()), allocator.line()))
                    .nest(2)
                    .group(),
            );
        }

        let fields = self.0.fields();
        if fields.len() != 0 {
            builder = builder
                .append(allocator.space())
                .append(allocator.text("{"))
                .append(
                    allocator
                        .hardline()
                        .append(FieldSequence::new(fields, self.1))
                        .nest(2),
                )
                .append(allocator.hardline())
                .append(allocator.text("}"));
        }

        builder
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<InputValueDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let description = self
            .0
            .description()
            .map(|description| {
                allocator
                    .nil()
                    .append(self.with_node(description))
                    .append(allocator.hardline())
            })
            .unwrap_or(allocator.softline_());

        let mut value_builder = allocator
            .text(self.0.name())
            .append(allocator.text(": "))
            .append(self.with_node(self.0.ty()));

        if let Some(value) = self.0.default_value() {
            value_builder = value_builder
                .append(allocator.space())
                .append(allocator.text("="))
                .append(allocator.space())
                .append(self.with_node(value));
        }

        let mut directives = self.0.directives().peekable();
        if directives.peek().is_some() {
            let directives_pretty = allocator
                .line()
                .append(allocator.intersperse(directives.map(self.mapper()), allocator.line()))
                .nest(2)
                .group();

            value_builder = value_builder.append(directives_pretty);
        }

        description.append(value_builder)
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<DirectiveDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(self.with_node(description))
                .append(allocator.hardline());
        }

        builder = builder
            .append(allocator.text("directive "))
            .append("@")
            .append(self.0.name());

        let arguments = self.0.arguments();

        if arguments.len() != 0 {
            let arguments = allocator
                .line_()
                .append(ArgumentSequence::new(arguments, self.1))
                .nest(2)
                .append(allocator.line_())
                .parens()
                .group();

            builder = builder.append(arguments);
        }

        if self.0.is_repeatable() {
            builder = builder.append(allocator.text(" repeatable"))
        }

        builder
            .append(allocator.text(" on "))
            .append(allocator.intersperse(
                self.0.locations().map(|location| location.as_str()),
                allocator.text(" | "),
            ))
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
                .line_()
                .append(
                    allocator.intersperse(
                        arguments.map(self.mapper()),
                        allocator
                            .line_()
                            .append(allocator.nil().flat_alt(allocator.text(", "))),
                    ),
                )
                .nest(2)
                .append(allocator.line_())
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
            .append(allocator.text(": "))
            .append(self.with_node(self.0.value()))
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<Description<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        NodeDisplay(self.0.literal(), self.1).pretty(allocator)
    }
}
