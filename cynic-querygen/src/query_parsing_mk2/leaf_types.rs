//! Handles "leaf types" - i.e. enums & scalars that don't have any nested fields.
use std::rc::Rc;

use super::{
    inputs::InputObjectSet,
    normalisation::NormalisedDocument,
    types::{Enum, Scalar},
};
use crate::{
    schema::{ScalarTypeExt, TypeDefinition},
    Error, TypeIndex,
};

pub fn extract_leaf_types<'query, 'schema>(
    doc: &NormalisedDocument<'query, 'schema>,
    inputs: &InputObjectSet,
    type_index: &Rc<TypeIndex<'schema>>,
) -> Result<(Vec<Enum<'schema>>, Vec<Scalar<'schema>>), Error> {
    let mut leaf_type_names = Vec::new();
    leaf_type_names.extend(
        doc.selection_sets
            .iter()
            .flat_map(|selection_set| selection_set.leaf_type_names()),
    );
    leaf_type_names.extend(
        inputs
            .iter()
            .flat_map(|input_object| input_object.leaf_type_names()),
    );

    let mut enums = Vec::new();
    let mut scalars = Vec::new();

    for name in leaf_type_names {
        match type_index.lookup_type(&name) {
            Some(TypeDefinition::Scalar(scalar)) => {
                if scalar.is_builtin() {
                    continue;
                }
                scalars.push(Scalar(scalar.name));
            }
            Some(TypeDefinition::Enum(def)) => {
                enums.push(Enum { def: def.clone() });
            }
            Some(_) => {}
            None => return Err(Error::UnknownType(name.to_string())),
        }
    }

    Ok((enums, scalars))
}
