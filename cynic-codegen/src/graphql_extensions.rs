/// Some extension traits for graphql_parser types.
use graphql_parser::schema::{Document, Field, InputValue};
use std::collections::HashSet;

use crate::{StructField, TypeIndex, TypePath};

pub trait FieldExt {
    fn required_arguments(&self) -> Vec<InputValue>;
    fn optional_arguments(&self) -> Vec<InputValue>;
}

impl FieldExt for Field {
    fn required_arguments(&self) -> Vec<InputValue> {
        self.arguments
            .iter()
            .filter(|arg| {
                // Note: We're passing an empty TypeIndex in here, but that's OK as
                // we only want to know if things are required
                StructField::from_input_value(arg, TypePath::empty(), &TypeIndex::empty())
                    .is_required()
            })
            .map(|a| a.clone())
            .collect()
    }

    fn optional_arguments(&self) -> Vec<InputValue> {
        self.arguments
            .iter()
            .filter(|arg| {
                // Note: We're passing an empty TypeIndex in here, but that's OK as
                // we only want to know if things are required
                !StructField::from_input_value(arg, TypePath::empty(), &TypeIndex::empty())
                    .is_required()
            })
            .map(|a| a.clone())
            .collect()
    }
}
