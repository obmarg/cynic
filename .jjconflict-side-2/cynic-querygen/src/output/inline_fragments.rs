use std::fmt::Write;

use crate::output::attr_output::Attributes;

use super::indented;

#[derive(Debug, PartialEq, Eq)]
pub struct InlineFragments {
    // TODO: Should this be a string?
    pub inner_type_names: Vec<String>,
    pub target_type: String,
    pub variable_struct_name: Option<String>,
    pub schema_name: Option<String>,

    pub name: String,
}

impl std::fmt::Display for InlineFragments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        writeln!(f, "#[derive(cynic::InlineFragments, Debug)]")?;

        let mut attributes = Attributes::new("cynic");
        if self.target_type != self.name {
            attributes.push(format!("graphql_type = \"{}\"", self.target_type));
        }

        if let Some(name) = &self.variable_struct_name {
            attributes.push(format!("variables = \"{name}\""));
        }

        if let Some(schema_name) = &self.schema_name {
            attributes.push(format!("schema = \"{schema_name}\""))
        }

        write!(f, "{attributes}")?;
        writeln!(f, "pub enum {} {{", self.name)?;
        for inner_type in &self.inner_type_names {
            writeln!(indented(f, 4), "{}({}),", inner_type, inner_type)?;
        }

        writeln!(indented(f, 4), "#[cynic(fallback)]\nUnknown")?;

        writeln!(f, "}}")
    }
}
