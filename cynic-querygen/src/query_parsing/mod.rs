use std::rc::Rc;

mod arguments;
pub mod inputs;
mod leaf_types;
pub mod normalisation;
mod parser;
mod sorting;
mod value;

use arguments::ArgumentStructDetails;
use parser::Document;

pub use normalisation::Variable;
pub use value::{LiteralContext, TypedValue};

use crate::{naming::Namer, output::Output, Error, TypeIndex};

pub fn parse_query_document<'text>(
    doc: &Document<'text>,
    type_index: &Rc<TypeIndex<'text>>,
) -> Result<Output<'text, 'text>, Error> {
    let normalised = normalisation::normalise(doc, type_index)?;
    let input_objects_raw = inputs::extract_input_objects(&normalised)?;

    let (mut enums, mut scalars) = leaf_types::extract_leaf_types(&normalised, &input_objects_raw)?;

    enums.sort_by_key(|e| e.name);
    scalars.sort_by_key(|s| s.0);

    let mut namers = Namers::new();

    let arg_struct_details = arguments::build_argument_structs(&normalised);

    for operation in &normalised.operations {
        let operation_name = operation.name.unwrap_or("UnnamedQuery");

        namers
            .selection_sets
            .force_name(&operation.root, operation_name);

        arg_struct_details.force_name_argument_struct_for(
            &operation.root,
            format!("{}Arguments", operation_name),
        );
    }

    let query_fragments = sorting::topological_sort(normalised.selection_sets.iter().cloned())
        .into_iter()
        .map(|selection| make_query_fragment(selection, &mut namers, &arg_struct_details))
        .collect::<Vec<_>>();

    let inline_fragments = normalised
        .inline_fragments
        .iter()
        .map(|fragment| make_inline_fragments(fragment.clone(), &mut namers, &arg_struct_details))
        .collect::<Vec<_>>();

    let input_objects = sorting::topological_sort(input_objects_raw.clone().into_iter())
        .into_iter()
        .map(make_input_object)
        .collect::<Vec<_>>();

    Ok(Output {
        query_fragments,
        inline_fragments,
        input_objects,
        enums,
        scalars,
        argument_structs: arg_struct_details.argument_structs(),
        normalised_document: normalised,
        input_objects_raw,
    })
}

fn make_query_fragment<'text>(
    selection: Rc<normalisation::SelectionSet<'text, 'text>>,
    namers: &mut Namers<'text>,
    argument_struct_details: &ArgumentStructDetails<'text, 'text>,
) -> crate::output::QueryFragment<'text, 'text> {
    use crate::output::query_fragment::{
        FieldArgument, OutputField, QueryFragment, RustOutputFieldType,
    };
    use normalisation::{Field, Selection};

    QueryFragment {
        fields: selection
            .selections
            .iter()
            .map(|selection| match selection {
                Selection::Field(field) => {
                    let schema_field = &field.schema_field;

                    let inner_type_name = match &field.field {
                        Field::Leaf => schema_field.value_type.inner_name().to_string(),
                        Field::Composite(ss) => namers.selection_sets.name_subject(ss),
                        Field::InlineFragments(fragments) => {
                            namers.inline_fragments.name_subject(fragments)
                        }
                    };

                    OutputField {
                        name: field.alias.unwrap_or(schema_field.name),
                        rename: field.alias.map(|_| schema_field.name),
                        field_type: RustOutputFieldType::from_schema_type(
                            &schema_field.value_type,
                            Some(inner_type_name),
                        ),
                        arguments: field
                            .arguments
                            .iter()
                            .map(|(name, value)| -> Result<FieldArgument, Error> {
                                Ok(FieldArgument::new(name.clone(), value.clone()))
                            })
                            .collect::<Result<Vec<_>, _>>()
                            .unwrap(),
                    }
                }
            })
            .collect(),
        argument_struct_name: argument_struct_details.argument_name_for_selection(&selection),

        name: namers.selection_sets.name_subject(&selection),
        target_type: selection.target_type.name().to_string(),
    }
}

fn make_inline_fragments<'text>(
    inline_fragments: Rc<normalisation::InlineFragments<'text, 'text>>,
    namers: &mut Namers<'text>,
    argument_struct_details: &ArgumentStructDetails<'text, 'text>,
) -> crate::output::InlineFragments {
    crate::output::InlineFragments {
        inner_type_names: inline_fragments
            .inner_selections
            .iter()
            .map(|s| namers.selection_sets.name_subject(s))
            .collect(),
        target_type: inline_fragments.abstract_type.name().to_string(),
        // Note: we just look for the first selection set with an argument struct.
        // Think it might be possible that there are two different argument structs within
        // and this will fall over in that case.  But it should be good enough for a first
        // pass
        argument_struct_name: inline_fragments
            .inner_selections
            .iter()
            .map(|selection| argument_struct_details.argument_name_for_selection(selection))
            .flatten()
            .next(),
        name: namers.inline_fragments.name_subject(&inline_fragments),
    }
}

fn make_input_object(input: Rc<inputs::InputObject>) -> crate::output::InputObject {
    crate::output::InputObject {
        name: input.schema_type.name.to_string(),
        fields: input.fields.clone(),
    }
}

#[derive(Debug)]
struct Namers<'text> {
    selection_sets: Namer<Rc<normalisation::SelectionSet<'text, 'text>>>,
    inline_fragments: Namer<Rc<normalisation::InlineFragments<'text, 'text>>>,
}

impl<'text> Namers<'text> {
    pub fn new() -> Self {
        Namers {
            selection_sets: Namer::new(),
            inline_fragments: Namer::new(),
        }
    }
}
