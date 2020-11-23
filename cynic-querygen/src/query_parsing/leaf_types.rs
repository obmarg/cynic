//! Handles "leaf types" - i.e. enums & scalars that don't have any nested fields.

use super::{inputs::InputObjectSet, normalisation::NormalisedDocument, types::Scalar};
use crate::{
    schema::{EnumDetails, InputType, OutputType},
    Error,
};

pub fn extract_leaf_types<'query, 'schema>(
    doc: &NormalisedDocument<'query, 'schema>,
    inputs: &InputObjectSet<'schema>,
) -> Result<(Vec<EnumDetails<'schema>>, Vec<Scalar<'schema>>), Error> {
    let mut enums = Vec::new();
    let mut scalars = Vec::new();

    let leaf_output_types = doc
        .selection_sets
        .iter()
        .flat_map(|selection_set| selection_set.leaf_output_types());

    for type_ref in leaf_output_types {
        match type_ref.lookup()? {
            OutputType::Scalar(s) => {
                if s.is_builtin() {
                    continue;
                }
                scalars.push(Scalar(s.name));
            }
            OutputType::Enum(en) => {
                enums.push(en.clone());
            }
            _ => {}
        }
    }

    let mut input_types = Vec::new();
    input_types.extend(
        doc.selection_sets
            .iter()
            .flat_map(|selection_set| selection_set.required_input_types()),
    );
    input_types.extend(
        inputs
            .iter()
            .flat_map(|input_object| input_object.required_input_types()),
    );

    for type_ref in input_types {
        match type_ref.lookup()? {
            InputType::Scalar(s) => {
                if s.is_builtin() {
                    continue;
                }
                scalars.push(Scalar(s.name));
            }
            InputType::Enum(en) => {
                enums.push(en.clone());
            }
            _ => {}
        }
    }

    Ok((enums, scalars))
}
