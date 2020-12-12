use std::rc::Rc;

mod arguments;
mod inputs;
mod leaf_types;
mod naming;
mod normalisation;
mod output;
mod parser;
mod sorting;
mod value;

use arguments::ArgumentStructDetails;
use naming::Namer;
use output::Output;
use parser::Document;

use crate::{Error, TypeIndex};

pub fn parse_query_document<'text>(
    doc: &Document<'text>,
    type_index: &Rc<TypeIndex<'text>>,
) -> Result<Output<'text, 'text>, Error> {
    let normalised = normalisation::normalise(doc, type_index)?;
    let input_objects = inputs::extract_input_objects(&normalised)?;

    let (mut enums, mut scalars) = leaf_types::extract_leaf_types(&normalised, &input_objects)?;

    enums.sort_by_key(|e| e.name);
    scalars.sort_by_key(|s| s.0);

    let mut query_namer = Namer::new();

    let arg_struct_details = arguments::build_argument_structs(&normalised);

    for operation in &normalised.operations {
        let operation_name = operation.name.unwrap_or("UnnamedQuery");

        query_namer.force_name(&operation.root, operation_name);
        arg_struct_details.force_name_argument_struct_for(
            &operation.root,
            format!("{}Arguments", operation_name),
        );
    }

    let query_fragments = sorting::topological_sort(normalised.selection_sets.iter().cloned())
        .into_iter()
        .map(|selection| make_query_fragment(selection, &mut query_namer, &arg_struct_details))
        .collect::<Result<Vec<_>, _>>()?;

    let input_objects = sorting::topological_sort(input_objects.into_iter())
        .into_iter()
        .map(make_input_object)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Output {
        query_fragments,
        input_objects,
        enums,
        scalars,
        argument_structs: arg_struct_details.argument_structs(),
    })
}

fn make_query_fragment<'text>(
    selection: Rc<normalisation::SelectionSet<'text, 'text>>,
    namer: &mut Namer<Rc<normalisation::SelectionSet<'text, 'text>>>,
    argument_struct_details: &ArgumentStructDetails<'text, 'text, '_>,
) -> Result<output::QueryFragment<'text, 'text>, Error> {
    use normalisation::{Field, Selection};
    use output::{FieldArgument, OutputField, QueryFragment, RustOutputFieldType};

    Ok(QueryFragment {
        fields: selection
            .selections
            .iter()
            .map(|selection| match selection {
                Selection::Field(field) => {
                    let schema_field = &field.schema_field;

                    let inner_type_name = match &field.field {
                        Field::Leaf => schema_field.value_type.inner_name().to_string(),
                        Field::Composite(ss) => namer.name_subject(ss),
                    };

                    OutputField {
                        name: schema_field.name,
                        field_type: RustOutputFieldType::from_schema_type(
                            &schema_field.value_type,
                            inner_type_name,
                        ),
                        arguments: field
                            .arguments
                            .iter()
                            .map(|(name, value)| -> Result<FieldArgument, Error> {
                                Ok(FieldArgument::new(name, value.clone()))
                            })
                            .collect::<Result<Vec<_>, _>>()
                            .unwrap(),
                    }
                }
            })
            .collect(),
        argument_struct_name: argument_struct_details.argument_name_for_selection(&selection),

        name: namer.name_subject(&selection),
        target_type: selection.target_type.name().to_string(),
    })
}

fn make_input_object<'text>(input: Rc<inputs::InputObject>) -> Result<output::InputObject, Error> {
    Ok(output::InputObject {
        name: input.schema_type.name.to_string(),
        fields: input.fields.clone(),
    })
}
