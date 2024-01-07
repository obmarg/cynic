use std::fmt::{Display, Write};

use pretty::{Arena, BoxAllocator, DocAllocator, Pretty};

use crate::ast::{
    ids::{InputObjectDefinitionId, ObjectDefinitionId},
    AstReader, Definition,
};

impl crate::Ast {
    pub fn to_sdl(&self) -> String {
        let mut output = String::new();

        for definition in self.reader().definitions() {
            match definition {
                Definition::Schema(_) => {}
                Definition::Object(object) => {
                    write!(&mut output, "{}", NodeDisplay(object)).ok();
                }
                Definition::InputObject(_) => todo!(),
            }
        }

        output
    }
}

impl<'a> Pretty<'a, BoxAllocator> for NodeDisplay<'a, ObjectDefinitionId> {
    fn pretty(self, allocator: &'a BoxAllocator) -> pretty::DocBuilder<'a, BoxAllocator, ()> {
        pretty::docs![allocator, "type", self.0.name()]
    }
}

pub struct NodeDisplay<'a, T>(AstReader<'a, T>);

impl Display for NodeDisplay<'_, ObjectDefinitionId> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "type {} {{", self.0.name())?;
        for field in self.0.fields() {
            writeln!(f, "  {}: {}", field.name(), "TODO")?;
        }
        writeln!(f, "}}")
    }
}

impl Display for NodeDisplay<'_, InputObjectDefinitionId> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "input {} {{", self.0.name())?;
        for field in self.0.fields() {
            writeln!(f, "  {}: {}", field.name(), "TODO")?;
        }
        writeln!(f, "}}")
    }
}
