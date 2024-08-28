use crate::{casings::CasingExt, output::attr_output::Attributes};
use std::fmt::Write;

use super::indented;
use crate::schema::EnumDetails;

pub struct Enum<'a> {
    pub details: EnumDetails<'a>,

    pub schema_name: Option<String>,
}

impl std::fmt::Display for Enum<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = self.details.name;

        writeln!(f, "#[derive(cynic::Enum, Clone, Copy, Debug)]")?;

        let mut attributes = Attributes::new("cynic");
        if type_name != type_name.to_pascal_case() {
            attributes.push(format!("graphql_type = \"{type_name}\""));
        }
        if let Some(schema_name) = &self.schema_name {
            attributes.push(format!("schema = \"{schema_name}\""));
        }

        write!(f, "{attributes}")?;
        writeln!(f, "pub enum {} {{", type_name.to_pascal_case())?;

        for variant in &self.details.values {
            let mut f = indented(f, 4);

            if variant.to_pascal_case().to_screaming_snake_case() != *variant {
                // If a pascal -> screaming snake casing roundtrip is not lossless
                // we need to explicitly rename this variant
                writeln!(f, "#[cynic(rename = \"{}\")]", variant)?;
            }

            writeln!(f, "{},", variant.to_pascal_case())?;
        }
        writeln!(f, "}}")
    }
}
