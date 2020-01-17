/// Some extension traits for graphql_parser types.
use graphql_parser::schema::{Document, Field, InputValue};
use std::collections::HashSet;

use crate::{StructField, TypePath};

pub trait DocumentExt {
    fn scalar_names(&self) -> HashSet<String>;
}

impl DocumentExt for Document {
    fn scalar_names(&self) -> HashSet<String> {
        use graphql_parser::schema::{Definition, TypeDefinition};

        let mut scalar_names = HashSet::new();

        for definition in &self.definitions {
            match definition {
                Definition::TypeDefinition(TypeDefinition::Scalar(scalar)) => {
                    scalar_names.insert(scalar.name.clone());
                }
                _ => {}
            }
        }

        scalar_names
    }
}

pub trait FieldExt {
    fn required_arguments(&self) -> Vec<InputValue>;
    fn optional_arguments(&self) -> Vec<InputValue>;
}

impl FieldExt for Field {
    fn required_arguments(&self) -> Vec<InputValue> {
        self.arguments
            .iter()
            .filter(|arg| {
                // Note: We're passing an empty scalar_names in here, but that's OK as
                // we only want to know if things are required
                StructField::from_input_value(arg, TypePath::empty(), &HashSet::new()).is_required()
            })
            .map(|a| a.clone())
            .collect()
    }

    fn optional_arguments(&self) -> Vec<InputValue> {
        self.arguments
            .iter()
            .filter(|arg| {
                // Note: We're passing an empty scalar_names in here, but that's OK as
                // we only want to know if things are required
                !StructField::from_input_value(arg, TypePath::empty(), &HashSet::new())
                    .is_required()
            })
            .map(|a| a.clone())
            .collect()
    }
}
