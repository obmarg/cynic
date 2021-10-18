use std::fmt::Write;

use super::indented;

#[derive(Debug, PartialEq)]
pub struct InlineFragments {
    // TODO: Should this be a string?
    pub inner_type_names: Vec<String>,
    pub target_type: String,
    pub argument_struct_name: Option<String>,

    pub name: String,
}

impl std::fmt::Display for InlineFragments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        writeln!(f, "#[derive(cynic::InlineFragments, Debug)]")?;
        if self.target_type != self.name || self.argument_struct_name.is_some() {
            write!(f, "#[cynic(")?;
            if self.target_type != self.name {
                write!(f, "graphql_type = \"{}\"", self.target_type)?;
            }

            if let Some(name) = &self.argument_struct_name {
                if self.target_type != self.name {
                    write!(f, ", ")?;
                }
                write!(f, "argument_struct = \"{}\"", name)?;
            }
            writeln!(f, ")]",)?;
        }

        writeln!(f, "pub enum {} {{", self.name)?;
        for inner_type in &self.inner_type_names {
            writeln!(indented(f, 4), "{}({}),", inner_type, inner_type)?;
        }

        writeln!(f, "}}")
    }
}
