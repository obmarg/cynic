use crate::casings::CasingExt;

use crate::query_parsing::Variable;

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

    fn type_spec(&self) -> String {
        match self {
            VariablesStructField::Variable(var) => var.value_type.type_spec().to_string(),
            VariablesStructField::NestedStruct(type_name) => type_name.clone(),
        }
    }
}

impl std::fmt::Display for VariablesStruct<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use super::indented;
        use std::fmt::Write;

        writeln!(f, "#[derive(cynic::QueryVariables, Debug)]")?;
        writeln!(f, "pub struct {} {{", self.name)?;

        for field in &self.fields {
            write!(indented(f, 4), "{}", field)?;
        }
        writeln!(f, "}}")
    }
}

impl std::fmt::Display for VariablesStructField<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", super::Field::new(&self.name(), &self.type_spec()))
    }
}
