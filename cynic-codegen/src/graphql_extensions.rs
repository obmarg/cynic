/// Some extension traits for graphql_parser types.
use graphql_parser::schema::{Field, InputValue};
use std::collections::HashSet;

use crate::argument::Argument;

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
                Argument::from_input_value(arg, &HashSet::new()).is_required()
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
                !Argument::from_input_value(arg, &HashSet::new()).is_required()
            })
            .map(|a| a.clone())
            .collect()
    }
}
