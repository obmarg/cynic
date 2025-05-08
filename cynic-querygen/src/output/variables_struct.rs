use std::{borrow::Cow, collections::HashMap};

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

impl VariablesStructField<'_, '_> {
    fn name(&self) -> String {
        match self {
            VariablesStructField::Variable(var) => var.name.to_snake_case(),
            VariablesStructField::NestedStruct(type_name) => type_name.to_snake_case(),
        }
    }

    fn type_spec(&self, input_objects_need_lifetime: &HashMap<&str, bool>) -> TypeSpec<'_> {
        match self {
            VariablesStructField::Variable(var) => var.value_type.type_spec(
                false,
                input_objects_need_lifetime
                    .get(&*var.value_type.inner_name())
                    .copied()
                    .unwrap_or(false),
            ),
            VariablesStructField::NestedStruct(type_name) => TypeSpec {
                name: Cow::Borrowed(type_name),
                contains_lifetime_a: input_objects_need_lifetime
                    .get(type_name.as_str())
                    .copied()
                    .unwrap_or(false),
            },
        }
    }
}

pub struct VariablesStructForDisplay<'v, 'i, 'q, 's> {
    pub variables_struct: &'v VariablesStruct<'q, 's>,
    pub input_objects_need_lifetime: &'i HashMap<&'i str, bool>,
}

impl std::fmt::Display for VariablesStructForDisplay<'_, '_, '_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use {super::indented, std::fmt::Write};

        writeln!(f, "#[derive(cynic::QueryVariables, Debug)]")?;
        let type_specs: Vec<_> = self
            .variables_struct
            .fields
            .iter()
            .map(|f| f.type_spec(self.input_objects_need_lifetime))
            .collect();
        writeln!(
            f,
            "pub struct {}{} {{",
            self.variables_struct.name,
            TypeSpec::lifetime(&type_specs)
        )?;

        for (field, type_spec) in self.variables_struct.fields.iter().zip(type_specs) {
            write!(
                indented(f, 4),
                "{}",
                super::Field::new(&field.name(), &type_spec)
            )?;
        }
        writeln!(f, "}}")
    }
}
