//! Handles "leaf types" - i.e. enums & scalars that don't have any nested fields.

use super::{inputs::InputObjectSet, normalisation::NormalisedDocument, output::Scalar};
use crate::{
    schema::{EnumDetails, Type, TypeRef},
    Error,
};

pub fn extract_leaf_types<'query, 'schema>(
    doc: &NormalisedDocument<'query, 'schema>,
    inputs: &InputObjectSet<'schema>,
) -> Result<(Vec<EnumDetails<'schema>>, Vec<Scalar<'schema>>), Error> {
    let mut leaf_types = doc
        .selection_sets
        .iter()
        .flat_map(|selection_set| selection_set.leaf_output_types())
        .map(TypeRef::from)
        .collect::<Vec<_>>();

    leaf_types.extend(
        doc.selection_sets
            .iter()
            .flat_map(|selection_set| selection_set.required_input_types())
            .map(TypeRef::from),
    );
    leaf_types.extend(
        inputs
            .iter()
            .flat_map(|input_object| input_object.required_input_types())
            .map(TypeRef::from),
    );

    let mut enums = Vec::new();
    let mut scalars = Vec::new();

    for type_ref in leaf_types {
        match type_ref.lookup()? {
            Type::Scalar(s) => {
                if s.is_builtin() {
                    continue;
                }
                scalars.push(Scalar(s.name));
            }
            Type::Enum(en) => {
                enums.push(en.clone());
            }
            _ => {}
        }
    }

    Ok((enums, scalars))
}
