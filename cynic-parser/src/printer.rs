use std::alloc::alloc;

use pretty::{BoxAllocator, DocAllocator, Pretty};

use crate::ast::{
    ids::{
        ArgumentId, DirectiveDefinitionId, DirectiveId, EnumDefinitionId, EnumValueDefinitionId,
        FieldDefinitionId, InputObjectDefinitionId, InputValueDefinitionId, InterfaceDefinitionId,
        ObjectDefinitionId, ScalarDefinitionId, SchemaDefinitionId, TypeId, UnionDefinitionId,
        ValueId,
    },
    AstReader, Definition, TypeDefinition,
};

impl crate::Ast {
    pub fn to_sdl(&self) -> String {
        let allocator = BoxAllocator;

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

pub struct NodeDisplay<'a, T>(AstReader<'a, T>);

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, SchemaDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(description.to_string())
                .append(allocator.hardline());
        }

        builder = builder.append(allocator.text("schema"));

        let mut roots = self.0.root_operations().peekable();
        if roots.peek().is_some() {
            builder = builder
                .append(allocator.space())
                .append(allocator.text("{"))
                .append(allocator.hardline())
                .append("  ")
                .append(
                    allocator
                        .intersperse(
                            roots.map(|(kind, name)| {
                                allocator.text(kind.to_string()).append(": ").append(name)
                            }),
                            allocator.hardline(),
                        )
                        .align(),
                )
                .append(allocator.hardline())
                .append(allocator.text("}"))
        }

        builder
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, ScalarDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(description.to_string())
                .append(allocator.hardline());
        }

        let mut directives = self.0.directives().peekable();
        let mut directives_pretty = allocator.nil();
        if directives.peek().is_some() {
            directives_pretty = allocator
                .space()
                .append(allocator.intersperse(directives.map(NodeDisplay), allocator.space()));
        }

        builder
            .append(allocator.text(format!("scalar {}", self.0.name())))
            .append(directives_pretty)
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, ObjectDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(description.to_string())
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
            builder = builder
                .append(allocator.space())
                .append(allocator.intersperse(directives.map(NodeDisplay), allocator.line()));
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

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, FieldDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let mut arguments_pretty = allocator.nil();

        let mut arguments = self.0.arguments().peekable();

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

        let mut directives = self.0.directives().peekable();
        let mut directives_pretty = allocator.nil();
        if directives.peek().is_some() {
            directives_pretty = allocator
                .space()
                .append(allocator.intersperse(directives.map(NodeDisplay), allocator.space()));
        }

        allocator
            .text(self.0.name().to_string())
            .append(arguments_pretty)
            .append(allocator.text(":"))
            .append(allocator.space())
            .append(NodeDisplay(self.0.ty()))
            .append(directives_pretty)
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, InterfaceDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(description.to_string())
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
            builder = builder
                .append(allocator.space())
                .append(allocator.intersperse(directives.map(NodeDisplay), allocator.line()));
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

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, UnionDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(description.to_string())
                .append(allocator.hardline());
        }

        builder = builder.append(allocator.text(format!("union {}", self.0.name())));

        let mut directives = self.0.directives().peekable();
        if directives.peek().is_some() {
            builder = builder
                .append(allocator.space())
                .append(allocator.intersperse(directives.map(NodeDisplay), allocator.line()));
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

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, EnumDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(description.to_string())
                .append(allocator.hardline());
        }

        builder = builder.append(allocator.text(format!("enum {}", self.0.name())));

        let mut directives = self.0.directives().peekable();
        if directives.peek().is_some() {
            builder = builder
                .append(allocator.space())
                .append(allocator.intersperse(directives.map(NodeDisplay), allocator.line()));
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
                        .indent(2)
                        .align(),
                )
                .append(allocator.hardline())
                .append(allocator.text("}"));
        }

        builder
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, EnumValueDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let mut builder = allocator.text(self.0.value().to_string());

        let mut directives = self.0.directives().peekable();
        if directives.peek().is_some() {
            builder = builder
                .append(allocator.space())
                .append(allocator.intersperse(directives.map(NodeDisplay), allocator.line()));
        }

        builder
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, InputObjectDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(description.to_string())
                .append(allocator.hardline());
        }

        builder = builder.append(allocator.text(format!("input {}", self.0.name())));

        let mut directives = self.0.directives().peekable();
        if directives.peek().is_some() {
            builder = builder
                .append(allocator.space())
                .append(allocator.intersperse(directives.map(NodeDisplay), allocator.line()));
        }

        let mut fields = self.0.fields().peekable();
        if fields.peek().is_some() {
            let fields = allocator.concat(fields.map(NodeDisplay)).nest(2);

            builder = builder
                .append(allocator.space())
                .append(allocator.text("{"))
                .append(allocator.hardline())
                .append(allocator.text("  "))
                .append(fields)
                .append(allocator.text("}"));
        }

        builder
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, InputValueDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let description = self
            .0
            .description()
            .map(|description| {
                allocator
                    .hardline()
                    .append(allocator.text(description))
                    .append(allocator.hardline())
            })
            .unwrap_or(allocator.nil());

        let mut value_builder = allocator
            .text(self.0.name().to_string())
            .append(allocator.text(":"))
            .append(allocator.space())
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
                .append(allocator.intersperse(directives.map(NodeDisplay), allocator.space()));
        }

        description
            .append(value_builder.clone())
            .append(allocator.line_())
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, DirectiveDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let mut builder = allocator.nil();

        if let Some(description) = self.0.description() {
            builder = builder
                .append(description.to_string())
                .append(allocator.hardline());
        }

        builder = builder.append(allocator.text(format!("directive @{}", self.0.name())));

        let mut arguments = self.0.arguments().peekable();

        if arguments.peek().is_some() {
            let arguments = allocator
                .intersperse(arguments.map(NodeDisplay), comma_or_nil(allocator))
                .group();

            let arguments = arguments
                .clone()
                .nest(2)
                .parens()
                .flat_alt(arguments.parens());

            builder = builder.append(arguments).append(allocator.space())
        }

        if self.0.is_repeatable() {
            builder = builder
                .append(allocator.text("repeatable"))
                .append(allocator.space())
        }

        builder
            .append(allocator.text("on"))
            .append(allocator.space())
            .append(allocator.intersperse(
                self.0.locations().map(|location| location.to_string()),
                allocator.text(" | "),
            ))
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, TypeId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        allocator.text(self.0.to_string())
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, DirectiveId> {
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

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, ArgumentId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        allocator
            .text(self.0.name().to_string())
            .append(allocator.text(":"))
            .append(allocator.space())
            .append(NodeDisplay(self.0.value()))
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, ValueId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        match self.0.value() {
            crate::ast::ValueReader::Variable(name) => allocator.text(format!("${name}")),
            crate::ast::ValueReader::Int(value) => allocator.text(format!("{value}")),
            crate::ast::ValueReader::Float(value) => allocator.text(format!("{value}")),
            crate::ast::ValueReader::String(value) => allocator.text(value.to_string()),
            crate::ast::ValueReader::Boolean(value) => allocator.text(format!("{value}")),
            crate::ast::ValueReader::Null => allocator.text("null"),
            crate::ast::ValueReader::Enum(value) => allocator.text(value.to_string()),
            crate::ast::ValueReader::List(items) => brackets_and_maybe_indent(
                allocator
                    .intersperse(
                        items.into_iter().map(NodeDisplay),
                        allocator.text(",").append(allocator.line()),
                    )
                    .group()
                    .enclose(allocator.line_(), allocator.line_()),
            ),
            crate::ast::ValueReader::Object(items) => allocator
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

fn braces_and_maybe_indent(
    thing: pretty::DocBuilder<'_, BoxAllocator>,
) -> pretty::DocBuilder<'_, BoxAllocator> {
    thing.clone().nest(2).braces().flat_alt(thing.braces())
}
