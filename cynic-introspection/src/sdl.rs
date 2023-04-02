use std::fmt::{self, Write};

use crate::query::{NamedType, Schema, Type};

impl Schema {
    /// Returns a String of the SDL for this Schema
    pub fn to_sdl(&self) -> String {
        format!("{self}")
    }
}

impl fmt::Display for Schema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "schema {{")?;
        {
            let f = &mut indented(f);
            writeln!(f, "query: {}", self.query_type)?;
            if let Some(mutation_type) = &self.mutation_type {
                writeln!(f, "mutation: {}", mutation_type)?;
            }

            if let Some(subscription_type) = &self.subscription_type {
                writeln!(f, "subscription: {}", subscription_type)?;
            }
        }
        writeln!(f, "}}")?;

        for ty in &self.types {
            writeln!(f, "{ty}")?;
        }

        Ok(())
    }
}

impl fmt::Display for NamedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self
            .name
            .as_ref()
            .expect("NamedType should always have a name");

        write!(f, "{name}")
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self
            .name
            .as_ref()
            .expect("NamedType should always have a name");

        match self.kind {
            crate::TypeKind::Scalar => {
                writeln!(f, "scalar {name}")
            }
            crate::TypeKind::Interface => todo!(),
            crate::TypeKind::Union => todo!(),
            crate::TypeKind::Enum => todo!(),
            crate::TypeKind::Object => todo!(),
            crate::TypeKind::InputObject => todo!(),
            crate::TypeKind::List => {
                panic!("Found a list where we were expecting a named yype")
            }
            crate::TypeKind::NonNull => {
                panic!("Found a non-null where we were expecting a named yype")
            }
        }
    }
}

pub fn indented<D>(f: &mut D) -> indenter::Indented<'_, D> {
    indenter::indented(f).with_str("  ")
}
