use std::fmt::{Display, Write};

use crate::ast::{ids::ObjectDefinitionId, AstReader, Definition};

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
