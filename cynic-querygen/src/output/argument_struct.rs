use inflector::Inflector;
use std::rc::Rc;
use uuid::Uuid;

use crate::query_parsing::Variable;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArgumentStruct<'query, 'schema> {
    pub id: Uuid,
    pub target_type_name: String,
    pub fields: Vec<ArgumentStructField<'query, 'schema>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ArgumentStructField<'query, 'schema> {
    Variable(Variable<'query, 'schema>),
    NestedStruct(Rc<ArgumentStruct<'query, 'schema>>),
}

impl<'query, 'schema> ArgumentStructField<'query, 'schema> {
    pub fn name(&self) -> String {
        match self {
            ArgumentStructField::Variable(var) => var.name.to_string(),
            ArgumentStructField::NestedStruct(arg_struct) => arg_struct.name.to_snake_case(),
        }
    }

    pub fn type_spec(&self) -> String {
        match self {
            ArgumentStructField::Variable(var) => var.value_type.type_spec().to_string(),
            ArgumentStructField::NestedStruct(arg_struct) => arg_struct.name.clone(),
        }
    }
}

impl std::fmt::Display for ArgumentStruct<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use super::indented;
        use std::fmt::Write;

        writeln!(f, "#[derive(cynic::FragmentArguments, Debug)]")?;
        writeln!(f, "pub struct {} {{", self.name)?;

        for field in &self.fields {
            write!(indented(f, 4), "{}", field)?;
        }
        writeln!(f, "}}")
    }
}

impl std::fmt::Display for ArgumentStructField<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "pub {}: {},",
            self.name().to_snake_case(),
            self.type_spec()
        )
    }
}

impl<'query, 'schema> crate::naming::Nameable for Rc<ArgumentStruct<'query, 'schema>> {
    fn requested_name(&self) -> String {
        self.requested_name.clone()
    }
}
