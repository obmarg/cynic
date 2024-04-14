use pretty::{DocAllocator, Pretty};

use crate::type_system::*;

use super::escape_string;

type Allocator<'a> = pretty::Arena<'a>;

impl crate::TypeSystemDocument {
    pub fn to_sdl(&self) -> String {
        let allocator = pretty::Arena::new();

        let builder = allocator
            .intersperse(
                self.definitions().map(|definition| match definition {
                    Definition::Schema(reader) => NodeDisplay(reader).pretty(&allocator),
                    Definition::SchemaExtension(reader) => allocator
                        .text("extend")
                        .append(allocator.space())
                        .append(NodeDisplay(reader).pretty(&allocator)),
                    Definition::Type(TypeDefinition::Scalar(reader)) => {
                        NodeDisplay(reader).pretty(&allocator)
                    }
                    Definition::Type(TypeDefinition::Object(reader)) => {
                        NodeDisplay(reader).pretty(&allocator)
                    }
                    Definition::Type(TypeDefinition::Interface(reader)) => {
                        NodeDisplay(reader).pretty(&allocator)
                    }
                    Definition::Type(TypeDefinition::Union(reader)) => {
                        NodeDisplay(reader).pretty(&allocator)
                    }
                    Definition::Type(TypeDefinition::Enum(reader)) => {
                        NodeDisplay(reader).pretty(&allocator)
                    }
                    Definition::Type(TypeDefinition::InputObject(reader)) => {
                        NodeDisplay(reader).pretty(&allocator)
                    }
                    Definition::TypeExtension(TypeDefinition::Scalar(reader)) => allocator
                        .text("extend")
                        .append(allocator.space())
                        .append(NodeDisplay(reader).pretty(&allocator)),
                    Definition::TypeExtension(TypeDefinition::Object(reader)) => allocator
                        .text("extend")
                        .append(allocator.space())
                        .append(NodeDisplay(reader).pretty(&allocator)),
                    Definition::TypeExtension(TypeDefinition::Interface(reader)) => allocator
                        .text("extend")
                        .append(allocator.space())
                        .append(NodeDisplay(reader).pretty(&allocator)),
                    Definition::TypeExtension(TypeDefinition::Union(reader)) => allocator
                        .text("extend")
                        .append(allocator.space())
                        .append(NodeDisplay(reader).pretty(&allocator)),
                    Definition::TypeExtension(TypeDefinition::Enum(reader)) => allocator
                        .text("extend")
                        .append(allocator.space())
                        .append(NodeDisplay(reader).pretty(&allocator)),
                    Definition::TypeExtension(TypeDefinition::InputObject(reader)) => allocator
                        .text("extend")
                        .append(allocator.space())
                        .append(NodeDisplay(reader).pretty(&allocator)),
                    Definition::Directive(reader) => NodeDisplay(reader).pretty(&allocator),
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

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<SchemaDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(NodeDisplay(description))
                .append(allocator.hardline());
        }

        builder = builder.append(allocator.text("schema"));

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
                .append(NodeDisplay(description))
                .append(allocator.hardline());
        }

        let mut directives = self.0.directives().peekable();
        let mut directives_pretty = allocator.nil();
        if directives.peek().is_some() {
            directives_pretty = allocator
                .space()
                .append(allocator.intersperse(directives.map(NodeDisplay), allocator.softline()));
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
                .append(NodeDisplay(description))
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
                    .append(allocator.intersperse(directives.map(NodeDisplay), allocator.line()))
                    .nest(2)
                    .group(),
            );
        }

        let mut fields = self.0.fields().peekable();
        if fields.peek().is_some() {
            builder = builder
                .append(allocator.space())
                .append(allocator.text("{"))
                .append(
                    allocator
                        .hardline()
                        .append(
                            allocator.intersperse(fields.map(NodeDisplay), allocator.hardline()),
                        )
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
        let mut arguments_pretty = allocator.nil();

        let mut arguments = self.0.arguments().peekable();

        if arguments.peek().is_some() {
            arguments_pretty = allocator
                .line_()
                .append(
                    allocator.intersperse(arguments.map(NodeDisplay), comma_or_newline(allocator)),
                )
                .nest(2)
                .append(allocator.line_())
                .parens()
                .group();

            // builder = builder.append(arguments);
            // arguments_pretty = allocator
            //     .intersperse(arguments.map(NodeDisplay), comma_or_newline(allocator))
            //     .nest(2)
            //     .append(allocator.softline_())
            //     .parens()
            //     .group();
        }

        let mut directives = self.0.directives().peekable();
        let mut directives_pretty = allocator.nil();
        if directives.peek().is_some() {
            directives_pretty = allocator
                .softline()
                .append(allocator.intersperse(directives.map(NodeDisplay), allocator.line()))
                .nest(2)
                .group();

            // directives_pretty = allocator.space().append(directives_pretty);
        }

        allocator
            .text(self.0.name())
            .append(arguments_pretty)
            .append(allocator.text(":"))
            .append(allocator.space())
            .append(NodeDisplay(self.0.ty()))
            .append(directives_pretty)
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<InterfaceDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(NodeDisplay(description))
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
                    .append(allocator.intersperse(directives.map(NodeDisplay), allocator.line()))
                    .nest(2)
                    .group(),
            );
        }

        let mut fields = self.0.fields().peekable();
        if fields.peek().is_some() {
            builder = builder
                .append(allocator.space())
                .append(allocator.text("{"))
                .append(allocator.hardline())
                .append(allocator.text("  "))
                .append(
                    allocator
                        .intersperse(fields.map(NodeDisplay), allocator.hardline())
                        .align(),
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

        if let Some(description) = self.0.description() {
            builder = builder
                .append(NodeDisplay(description))
                .append(allocator.hardline());
        }

        builder = builder.append(allocator.text(format!("union {}", self.0.name())));

        let mut directives = self.0.directives().peekable();
        if directives.peek().is_some() {
            builder = builder.append(
                allocator
                    .line()
                    .append(allocator.intersperse(directives.map(NodeDisplay), allocator.line()))
                    .nest(2)
                    .group(),
            );
        }

        let mut members = self.0.members().peekable();

        if members.peek().is_some() {
            let members = allocator
                .intersperse(members, allocator.line().append(allocator.text("| ")))
                .group();

            let members = members.clone().nest(2).flat_alt(members);

            builder = builder
                .append(allocator.space())
                .append(allocator.text("="))
                .append(allocator.space())
                .append(members)
        }

        builder
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<EnumDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(NodeDisplay(description))
                .append(allocator.hardline());
        }

        builder = builder.append(allocator.text(format!("enum {}", self.0.name())));

        let mut directives = self.0.directives().peekable();
        if directives.peek().is_some() {
            builder = builder.append(
                allocator
                    .line()
                    .append(allocator.intersperse(directives.map(NodeDisplay), allocator.line()))
                    .nest(2)
                    .group(),
            );
        }

        let values = self.0.values().collect::<Vec<_>>();

        if !values.is_empty() {
            builder = builder
                .append(allocator.space())
                .append(allocator.text("{"))
                .append(allocator.hardline())
                .append(
                    allocator
                        .intersperse(values.into_iter().map(NodeDisplay), allocator.hardline())
                        .indent(2),
                )
                .append(allocator.hardline())
                .append(allocator.text("}"));
        }

        builder
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<EnumValueDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut builder = allocator.text(self.0.value());

        let mut directives = self.0.directives().peekable();
        if directives.peek().is_some() {
            builder = builder
                .append(allocator.space())
                .append(allocator.intersperse(directives.map(NodeDisplay), allocator.softline()));
        }

        builder
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<InputObjectDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(NodeDisplay(description))
                .append(allocator.hardline());
        }

        builder = builder.append(allocator.text(format!("input {}", self.0.name())));

        let mut directives = self.0.directives().peekable();
        if directives.peek().is_some() {
            builder = builder.append(
                allocator
                    .line()
                    .append(allocator.intersperse(directives.map(NodeDisplay), allocator.line()))
                    .nest(2)
                    .group(),
            );
        }

        let mut fields = self.0.fields().peekable();
        if fields.peek().is_some() {
            builder = builder
                .append(allocator.space())
                .append(allocator.text("{"))
                .append(
                    allocator
                        .hardline()
                        .append(
                            allocator.intersperse(fields.map(NodeDisplay), allocator.hardline()),
                        )
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
                    .append(NodeDisplay(description))
                    .append(allocator.hardline())
            })
            .unwrap_or(allocator.softline_());

        let mut value_builder = allocator
            .text(self.0.name())
            .append(allocator.text(": "))
            .append(NodeDisplay(self.0.ty()));

        if let Some(value) = self.0.default_value() {
            value_builder = value_builder
                .append(allocator.space())
                .append(allocator.text("="))
                .append(allocator.space())
                .append(NodeDisplay(value));
        }

        let mut directives = self.0.directives().peekable();
        if directives.peek().is_some() {
            value_builder = value_builder
                .append(allocator.space())
                .append(allocator.intersperse(directives.map(NodeDisplay), allocator.softline()));
        }

        description.append(value_builder)
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<DirectiveDefinition<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(NodeDisplay(description))
                .append(allocator.hardline());
        }

        builder = builder
            .append(allocator.text("directive "))
            .append("@")
            .append(self.0.name());

        let mut arguments = self.0.arguments().peekable();

        if arguments.peek().is_some() {
            let arguments = allocator
                .line_()
                .append(
                    allocator.intersperse(arguments.map(NodeDisplay), comma_or_newline(allocator)),
                )
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
                .intersperse(arguments.map(NodeDisplay), comma_or_newline(allocator))
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
            .append(NodeDisplay(self.0.value()))
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<Value<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        match self.0 {
            crate::type_system::Value::Variable(name) => allocator.text(format!("${name}")),
            crate::type_system::Value::Int(value) => allocator.text(format!("{value}")),
            crate::type_system::Value::Float(value) => allocator.text(format!("{value}")),
            crate::type_system::Value::String(value) => {
                allocator.text(escape_string(value)).double_quotes()
            }
            crate::type_system::Value::BlockString(value) => allocator
                .text(value)
                .double_quotes()
                .double_quotes()
                .double_quotes(),
            crate::type_system::Value::Boolean(value) => allocator.text(format!("{value}")),
            crate::type_system::Value::Null => allocator.text("null"),
            crate::type_system::Value::Enum(value) => allocator.text(value),
            crate::type_system::Value::List(items) if items.is_empty() => {
                allocator.nil().brackets()
            }
            crate::type_system::Value::List(items) => allocator
                .line_()
                .append(
                    allocator.intersperse(
                        items.into_iter().map(NodeDisplay),
                        allocator
                            .line_()
                            .append(allocator.nil().flat_alt(allocator.text(", "))),
                    ),
                )
                .nest(2)
                .append(allocator.line_())
                .brackets()
                .group(),
            crate::type_system::Value::Object(items) if items.is_empty() => {
                allocator.nil().braces()
            }
            crate::type_system::Value::Object(items) => allocator
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

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<StringLiteral<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        match self.0.kind() {
            StringLiteralKind::String => allocator
                .text(escape_string(self.0.to_cow().as_ref()))
                .double_quotes(),
            StringLiteralKind::Block => allocator
                .text(self.0.raw_str())
                .double_quotes()
                .double_quotes()
                .double_quotes(),
        }
    }
}

fn comma_or_nil<'a>(allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>> {
    allocator
        .nil()
        .flat_alt(allocator.text(",").append(allocator.space()))
}

fn comma_or_newline<'a>(allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>> {
    allocator.line().flat_alt(allocator.text(", "))
}

fn parens_and_maybe_indent<'a>(
    thing: pretty::DocBuilder<'a, Allocator<'a>>,
) -> pretty::DocBuilder<'a, Allocator<'a>> {
    thing.clone().nest(2).parens().flat_alt(thing.parens())
}
