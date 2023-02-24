use std::borrow::Cow;

use crate::casings::CasingExt;

use crate::{query_parsing::Variable, schema::TypeSpec};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VariablesStruct<'query, 'schema> {
    name: String,
    fields: Vec<VariablesStructField<'query, 'schema>>,
}

impl<'query, 'schema> VariablesStruct<'query, 'schema> {
    pub fn new(name: String, fields: Vec<VariablesStructField<'query, 'schema>>) -> Self {
        VariablesStruct { name, fields }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VariablesStructField<'query, 'schema> {
    Variable(Variable<'query, 'schema>),
    NestedStruct(String),
}

impl<'query, 'schema> VariablesStructField<'query, 'schema> {
    fn name(&self) -> String {
        match self {
            VariablesStructField::Variable(var) => var.name.to_snake_case(),
            VariablesStructField::NestedStruct(type_name) => type_name.to_snake_case(),
        }
    }

    fn type_spec(&self) -> TypeSpec<'_> {
        match self {
            VariablesStructField::Variable(var) => var.value_type.type_spec(false, false),
            VariablesStructField::NestedStruct(type_name) => TypeSpec {
                name: Cow::Borrowed(type_name),
                contains_lifetime_a: false,
            },
        }
    }
}

impl std::fmt::Display for VariablesStruct<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use {super::indented, std::fmt::Write};

        writeln!(f, "#[derive(cynic::QueryVariables, Debug)]")?;
        let type_specs: Vec<_> = self.fields.iter().map(|f| f.type_spec()).collect();
        writeln!(
            f,
            "pub struct {}{} {{",
            self.name,
            TypeSpec::lifetime(&type_specs)
        )?;

        for (field, type_spec) in self.fields.iter().zip(type_specs) {
            write!(
                indented(f, 4),
                "{}",
                super::Field::new(&field.name(), &type_spec.name)
            )?;
        }
        writeln!(f, "}}")
    }
}
