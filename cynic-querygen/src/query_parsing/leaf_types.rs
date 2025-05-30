//! Handles "leaf types" - i.e. enums & scalars that don't have any nested fields.
use std::collections::HashSet;

use super::{inputs::InputObjects, normalisation::NormalisedDocument};
use crate::{
    output::Scalar,
    schema::{EnumDetails, Type, TypeRef},
    Error,
};

pub fn extract_leaf_types<'schema>(
    doc: &NormalisedDocument<'_, 'schema>,
    inputs: &InputObjects<'schema>,
) -> Result<(Vec<EnumDetails<'schema>>, Vec<Scalar<'schema>>), Error> {
    let mut leaf_types = doc
        .selection_sets
        .iter()
        .flat_map(|selection_set| selection_set.leaf_output_types())
        .map(TypeRef::from)
        .collect::<HashSet<_>>();

    leaf_types.extend(
        doc.operations
            .iter()
            .flat_map(|o| &o.variables)
            .map(|variables| variables.value_type.inner_ref().clone())
            .map(TypeRef::from),
    );

    leaf_types.extend(
        doc.selection_sets
            .iter()
            .flat_map(|selection_set| selection_set.required_input_types())
            .map(TypeRef::from),
    );
    leaf_types.extend(inputs.required_input_types().map(TypeRef::from));

    let mut enums = Vec::new();
    let mut scalars = Vec::new();

    for type_ref in leaf_types {
        match type_ref.lookup()? {
            Type::Scalar(s) => {
                if s.is_builtin() {
                    continue;
                }
                scalars.push(Scalar {
                    name: s.name,
                    schema_name: None,
                });
            }
            Type::Enum(en) => {
                enums.push(en.clone());
            }
            _ => {}
        }
    }

    Ok((enums, scalars))
}
