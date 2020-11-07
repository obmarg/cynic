mod inputs;
mod leaf_types;
mod normalisation;
mod sorting;
mod types;
mod value;

use value::Value;

use crate::{query::Document, Error, TypeIndex};

use std::rc::Rc;

pub fn parse_query_document<'text>(
    doc: &Document<'text>,
    type_index: &TypeIndex<'text>,
) -> Result<types::Output<'text, 'text>, Error> {
    let normalised = normalisation::normalise(doc, type_index)?;
    let input_objects = inputs::extract_input_objects(&normalised, type_index)?;

    let (enums, scalars) = leaf_types::extract_leaf_types(&normalised, &input_objects, type_index)?;

    let query_fragments = sorting::topological_sort(normalised.selection_sets.into_iter())
        .into_iter()
        .map(|selection_set| make_query_fragment(selection_set, &type_index))
        .collect::<Result<Vec<_>, _>>()?;

    let input_objects = sorting::topological_sort(input_objects.into_iter())
        .into_iter()
        .map(|obj| make_input_object(obj, type_index))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(types::Output {
        query_fragments,
        input_objects,
        enums,
        scalars,
        // TODO: argument structs
        argument_structs: vec![],
    })
}

fn make_query_fragment<'text>(
    selection: Rc<normalisation::SelectionSet<'text>>,
    type_index: &TypeIndex<'text>,
) -> Result<types::QueryFragment<'text, 'text>, Error> {
    use crate::{schema::TypeDefinition, type_ext::TypeExt};
    use normalisation::Selection;
    use types::{Field, FieldArgument};

    let selection_type = type_index
        .lookup_type(&selection.target_type)
        .ok_or_else(|| Error::UnknownType(selection.target_type.clone()))?;

    let obj = if let TypeDefinition::Object(obj) = selection_type {
        obj
    } else {
        return Err(Error::ExpectedObject(selection.target_type.clone()));
    };

    Ok(types::QueryFragment {
        fields: selection
            .selections
            .iter()
            .map(|selection| {
                match selection {
                    Selection::Field(field) => {
                        // TODO: Stop unwrapping
                        let schema_field =
                            obj.fields.iter().find(|f| f.name == field.name).unwrap();

                        Field {
                            name: schema_field.name,
                            field_type: schema_field.field_type.clone(),
                            arguments: field
                                .arguments
                                .iter()
                                .map(|(name, value)| -> Result<FieldArgument, Error> {
                                    let argument = schema_field
                                        .arguments
                                        .iter()
                                        .find(|arg| &arg.name == name)
                                        .ok_or(Error::UnknownArgument(name.to_string()))?;

                                    let argument_type = type_index
                                        .lookup_type(argument.value_type.inner_name())
                                        .ok_or(Error::UnknownType(
                                            argument.value_type.inner_name().to_string(),
                                        ))?;

                                    Ok(FieldArgument::new(
                                        name,
                                        value.clone(),
                                        argument.clone(),
                                        argument_type.clone(),
                                    ))
                                })
                                .collect::<Result<Vec<_>, _>>()
                                .unwrap(),
                        }
                    }
                }
            })
            .collect(),
        argument_struct_name: None,
        name: selection.target_type.clone(),
        target_type: selection.target_type.clone(),
    })
}

fn make_input_object<'text>(
    input: Rc<inputs::InputObject>,
    type_index: &TypeIndex<'text>,
) -> Result<types::InputObject<'text>, Error> {
    use crate::{schema::TypeDefinition, type_ext::TypeExt};
    use normalisation::Selection;
    use types::{Field, FieldArgument};

    let input_type = type_index
        .lookup_type(&input.target_type)
        .ok_or_else(|| Error::UnknownType(input.target_type.clone()))?;

    let input_obj = if let TypeDefinition::InputObject(obj) = input_type {
        obj
    } else {
        return Err(Error::ExpectedInputObject(input.target_type.to_string()));
    };

    let mut fields = Vec::new();
    for (field_name, _) in &input.fields {
        fields.push(
            input_obj
                .fields
                .iter()
                .find(|f| f.name == field_name)
                .ok_or_else(|| Error::UnknownField(field_name.clone(), input.target_type.clone()))?
                .clone(),
        );
    }

    Ok(types::InputObject {
        name: input.target_type.clone(),
        fields,
    })
}
