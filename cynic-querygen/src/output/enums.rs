use inflector::Inflector;

use crate::schema::EnumDetails;

impl std::fmt::Display for EnumDetails<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = self.name;

        writeln!(f, "#[derive(cynic::Enum, Clone, Copy, Debug)]")?;
        writeln!(f, "#[cynic(")?;
        writeln!(f, "    graphql_type = \"{}\",", type_name)?;
        writeln!(f, "    rename_all = \"SCREAMING_SNAKE_CASE\"")?;
        writeln!(f, ")]")?;
        writeln!(f, "pub enum {} {{", type_name.to_pascal_case())?;

        for variant in &self.values {
            writeln!(f, "    {},", variant.to_pascal_case())?;
        }
        writeln!(f, "}}")
    }
}
