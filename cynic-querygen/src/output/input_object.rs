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
            let mut f = indented(f, 4);

            if field.name.to_snake_case().to_camel_case() != field.name {
                // If a snake -> camel casing roundtrip is not lossless
                // we need to explicitly rename this field
                writeln!(f, "#[cynic(rename = \"{}\")]", field.name)?;
            }

            writeln!(
                f,
                "pub {}: {},",
                field.name.to_snake_case(),
                field.type_spec()
            )?;
        }

        writeln!(f, "}}")
    }
}
