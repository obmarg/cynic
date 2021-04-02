use inflector::Inflector;
use std::fmt::Write;

use super::indented;
use crate::schema::EnumDetails;

impl std::fmt::Display for EnumDetails<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = self.name;

        writeln!(f, "#[derive(cynic::Enum, Clone, Copy, Debug)]")?;
        if type_name != type_name.to_pascal_case() {
            writeln!(f, "#[cynic(graphql_type = \"{}\")]", type_name)?;
        }
        writeln!(f, "pub enum {} {{", type_name.to_pascal_case())?;

        for variant in &self.values {
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
