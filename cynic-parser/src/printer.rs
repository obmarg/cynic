use std::fmt::{Display, Write};

use pretty::{Arena, BoxAllocator, DocAllocator, Pretty};

use crate::ast::{
    ids::{
        ArgumentId, DirectiveId, EnumDefinitionId, EnumValueDefinitionId, FieldDefinitionId,
        InputObjectDefinitionId, InputValueDefinitionId, InterfaceDefinitionId, ObjectDefinitionId,
        ScalarDefinitionId, SchemaDefinitionId, TypeId, UnionDefinitionId, ValueId,
    },
    AstDefinition, AstReader, Definition,
};

impl crate::Ast {
    pub fn to_sdl(&self) -> String {
        let allocator = BoxAllocator;

        let builder = allocator
            .concat(self.definitions().map(|definition| match definition {
                Definition::Schema(reader) => NodeDisplay(reader).pretty(&allocator),
                Definition::Scalar(reader) => NodeDisplay(reader).pretty(&allocator),
                Definition::Object(reader) => NodeDisplay(reader).pretty(&allocator),
                Definition::Interface(reader) => NodeDisplay(reader).pretty(&allocator),
                Definition::Union(reader) => NodeDisplay(reader).pretty(&allocator),
                Definition::Enum(reader) => NodeDisplay(reader).pretty(&allocator),
                Definition::InputObject(reader) => NodeDisplay(reader).pretty(&allocator),
            }))
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
        let mut builder = allocator.text("schema");
        let roots = self.0.root_operations().collect::<Vec<_>>();

        if !roots.is_empty() {
            builder = builder
                .append(allocator.space())
                .append(allocator.text("{"))
                .append(allocator.hardline())
                .append("  ")
                .append(
                    allocator
                        .intersperse(
                            roots.into_iter().map(|(kind, name)| {
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
        allocator
            .text(format!("scalar {}", self.0.name()))
            .append(allocator.space())
            .append(allocator.intersperse(self.0.directives().map(NodeDisplay), allocator.line()))
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, ObjectDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let mut builder = allocator
            .text(format!("type {}", self.0.name()))
            .append(allocator.space());

        let interfaces = self.0.implements_interfaces().collect::<Vec<_>>();

        if !interfaces.is_empty() {
            builder = builder
                .append(allocator.text("implements"))
                .append(allocator.space())
                .append(allocator.intersperse(interfaces, " & "))
                .append(allocator.space());
        }

        builder
            .append(allocator.intersperse(self.0.directives().map(NodeDisplay), allocator.line()))
            .append(allocator.space())
            .append(allocator.text("{"))
            .append(allocator.hardline())
            .append(allocator.text("  "))
            .append(
                allocator
                    .intersperse(self.0.fields().map(NodeDisplay), allocator.hardline())
                    .align(),
            )
            .append(allocator.hardline())
            .append(allocator.text("}"))
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, FieldDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        allocator
            .text(self.0.name().to_string())
            .append(allocator.text(":"))
            .append(allocator.space())
            .append(NodeDisplay(self.0.ty()))
            .append(allocator.intersperse(self.0.directives().map(NodeDisplay), " "))
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, InterfaceDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let mut builder = allocator
            .text(format!("interface {}", self.0.name()))
            .append(allocator.space());

        let interfaces = self.0.implements_interfaces().collect::<Vec<_>>();

        if !interfaces.is_empty() {
            builder = builder
                .append(allocator.text("implements"))
                .append(allocator.space())
                .append(allocator.intersperse(interfaces, " & "))
                .append(allocator.space());
        }

        builder
            .append(allocator.intersperse(self.0.directives().map(NodeDisplay), allocator.line()))
            .append(allocator.space())
            .append(allocator.text("{"))
            .append(allocator.hardline())
            .append(allocator.text("  "))
            .append(
                allocator
                    .intersperse(self.0.fields().map(NodeDisplay), allocator.hardline())
                    .align(),
            )
            .append(allocator.hardline())
            .append(allocator.text("}"))
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, UnionDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let mut builder = allocator.text(format!("union {}", self.0.name()));

        let directives = self.0.directives().collect::<Vec<_>>();
        if !directives.is_empty() {
            builder = builder
                .append(allocator.space())
                .append(
                    allocator.intersperse(self.0.directives().map(NodeDisplay), allocator.line()),
                )
                .append(allocator.space());
        }

        let members = self.0.members().collect::<Vec<_>>();

        if !members.is_empty() {
            builder = builder
                .append(allocator.text("="))
                .append(allocator.space())
                .append(allocator.intersperse(members, allocator.text(" | ")))
                .append(allocator.hardline());
        }

        builder
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, EnumDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let mut builder = allocator
            .text(format!("enum {}", self.0.name()))
            .append(allocator.space());

        let directives = self.0.directives().collect::<Vec<_>>();
        if !directives.is_empty() {
            builder = builder
                .append(
                    allocator.intersperse(self.0.directives().map(NodeDisplay), allocator.line()),
                )
                .append(allocator.space());
        }

        let values = self.0.values().collect::<Vec<_>>();

        if !values.is_empty() {
            builder = builder
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

        let directives = self.0.directives().collect::<Vec<_>>();
        if !directives.is_empty() {
            builder = builder
                .append(
                    allocator.intersperse(self.0.directives().map(NodeDisplay), allocator.line()),
                )
                .append(allocator.space());
        }

        builder
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, InputObjectDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        allocator
            .text(format!("input {}", self.0.name()))
            .append(allocator.space())
            .append(allocator.intersperse(self.0.directives().map(NodeDisplay), allocator.line()))
            .append(allocator.space())
            .append(allocator.text("{"))
            .append(allocator.hardline())
            .append(allocator.text("  "))
            .append(
                allocator
                    .intersperse(self.0.fields().map(NodeDisplay), allocator.hardline())
                    .align(),
            )
            .append(allocator.hardline())
            .append(allocator.text("}"))
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, InputValueDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        let mut builder = allocator
            .text(self.0.name().to_string())
            .append(allocator.text(":"))
            .append(allocator.space())
            .append(NodeDisplay(self.0.ty()));

        if let Some(value) = self.0.default_value() {
            builder = builder
                .append(allocator.space())
                .append(allocator.text("="))
                .append(allocator.space())
                .append(NodeDisplay(value));
        }

        builder.append(allocator.intersperse(self.0.directives().map(NodeDisplay), " "))
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

        let arguments = self.0.arguments().collect::<Vec<_>>();
        if !arguments.is_empty() {
            builder = builder.append(
                allocator
                    .intersperse(arguments.into_iter().map(NodeDisplay), ", ")
                    .parens(),
            );
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
            crate::ast::ValueReader::List(items) => allocator
                .intersperse(items.into_iter().map(NodeDisplay), ",")
                .parens(),
            crate::ast::ValueReader::Object(items) => allocator
                .intersperse(
                    items.into_iter().map(|(name, value)| {
                        allocator
                            .text(name)
                            .append(allocator.text(":"))
                            .append(NodeDisplay(value))
                    }),
                    ",",
                )
                .braces(),
        }
    }
}
