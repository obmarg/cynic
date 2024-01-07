use std::fmt::{Display, Write};

use pretty::{Arena, BoxAllocator, DocAllocator, Pretty};

use crate::ast::{
    ids::{
        FieldDefinitionId, InputObjectDefinitionId, InputValueDefinitionId, ObjectDefinitionId,
        TypeId,
    },
    AstReader, Definition,
};

impl crate::Ast {
    pub fn to_sdl(&self) -> String {
        let allocator = BoxAllocator;

        let builder = allocator
            .concat(
                self.reader()
                    .definitions()
                    .map(|definition| match definition {
                        Definition::Schema(_) => todo!(),
                        Definition::Object(object) => NodeDisplay(object).pretty(&allocator),
                        Definition::InputObject(_) => todo!(),
                    }),
            )
            .pretty(&allocator);

        #[allow(clippy::needless_borrow)] // This doesn't work without the borrow :|
        {
            format!("{}", (&*builder).pretty(80))
        }
    }
}

pub struct NodeDisplay<'a, T>(AstReader<'a, T>);

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, ObjectDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        allocator
            .text(format!("type {} {{", self.0.name()))
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
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, InputObjectDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        allocator
            .text(format!("type {} {{", self.0.name()))
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
        allocator
            .text(self.0.name().to_string())
            .append(allocator.text(":"))
            .append(allocator.space())
            .append(NodeDisplay(self.0.ty()))
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, TypeId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        allocator.text(self.0.to_string())
    }
}
