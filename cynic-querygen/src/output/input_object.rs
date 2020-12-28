use std::fmt::Write;

use inflector::Inflector;

use super::indented;
use crate::schema::InputField;

#[derive(Debug, PartialEq)]
pub struct InputObject<'schema> {
    pub name: String,
    pub fields: Vec<InputField<'schema>>,
}

impl std::fmt::Display for InputObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#[derive(cynic::InputObject, Debug)]")?;
        writeln!(
            f,
            "#[cynic(graphql_type = \"{}\", rename_all=\"camelCase\")]",
            self.name
        )?;
        writeln!(f, "pub struct {} {{", self.name)?;

        for field in &self.fields {
            writeln!(
                indented(f, 4),
                "pub {}: {},",
                field.name.to_snake_case(),
                field.type_spec()
            )?;
        }

        writeln!(f, "}}")
    }
}
