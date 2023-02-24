use std::fmt::Write;

use crate::{casings::CasingExt, schema};

use super::indented;

#[derive(Debug, PartialEq, Eq)]
pub struct InputObject<'schema> {
    pub name: String,
    pub fields: Vec<InputObjectField<'schema>>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputObjectField<'schema> {
    pub schema_field: schema::InputField<'schema>,
    pub needs_boxed: bool,
}

impl<'schema> InputObjectField<'schema> {
    pub fn type_spec(&self) -> schema::TypeSpec {
        self.schema_field
            .value_type
            .type_spec(self.needs_boxed, false)
    }
}

impl std::fmt::Display for InputObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#[derive(cynic::InputObject, Debug)]")?;
        if self.name != self.name.to_pascal_case() {
            writeln!(f, "#[cynic(graphql_type = \"{}\")]", self.name)?;
        }
        let type_specs: Vec<_> = self.fields.iter().map(|f| f.type_spec()).collect();
        writeln!(
            f,
            "pub struct {}{} {{",
            self.name.to_pascal_case(),
            schema::TypeSpec::lifetime(&type_specs)
        )?;

        for (field, type_spec) in self.fields.iter().zip(type_specs) {
            let mut f = indented(f, 4);

            let name = field.schema_field.name.to_snake_case();
            let mut output = super::Field::new(&name, &type_spec.name);

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
