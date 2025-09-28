mod variant;

use std::fmt::Write;

use crate::{casings::CasingExt, output::attr_output::Attributes, schema};

use super::indented;

#[derive(Debug, PartialEq, Eq)]
pub struct InputObject<'schema> {
    pub name: String,
    pub fields: Vec<InputObjectField<'schema>>,
    pub schema_name: Option<String>,
    pub is_oneof: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputObjectField<'schema> {
    pub schema_field: schema::InputField<'schema>,
    pub type_spec: schema::TypeSpec<'static>,
}

impl std::fmt::Display for InputObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#[derive(cynic::InputObject, Debug)]")?;

        let type_name = &self.name;

        let mut attributes = Attributes::new("cynic");
        if *type_name != type_name.to_pascal_case() {
            attributes.push(format!("graphql_type = \"{type_name}\""));
        }
        if let Some(schema_name) = &self.schema_name {
            attributes.push(format!("schema = \"{schema_name}\""));
        }

        write!(f, "{attributes}")?;
        if self.is_oneof {
            writeln!(
                f,
                "pub enum {}{} {{",
                self.name.to_pascal_case(),
                schema::TypeSpec::lifetime(self.fields.iter().map(|f| &f.type_spec))
            )?;

            for field in self.fields.iter() {
                let mut f = indented(f, 4);

                let name = field.schema_field.name.to_pascal_case();
                let mut output = variant::Variant::new(&name, &field.type_spec);

                if name.to_snake_case() != field.schema_field.name {
                    // If a snake -> pascal casing roundtrip is not lossless
                    // we need to explicitly rename this field
                    output.add_rename(field.schema_field.name);
                }

                write!(f, "{}", output)?;
            }

            writeln!(f, "}}")
        } else {
            writeln!(
                f,
                "pub struct {}{} {{",
                self.name.to_pascal_case(),
                schema::TypeSpec::lifetime(self.fields.iter().map(|f| &f.type_spec))
            )?;

            for field in self.fields.iter() {
                let mut f = indented(f, 4);

                let name = field.schema_field.name.to_snake_case();
                let mut output = super::Field::new(&name, &field.type_spec);

                if name.to_camel_case() != field.schema_field.name {
                    // If a snake -> camel casing roundtrip is not lossless
                    // we need to explicitly rename this field
                    output.add_rename(field.schema_field.name);
                }

                write!(f, "{}", output)?;
            }

            writeln!(f, "}}")
        }
    }
}
