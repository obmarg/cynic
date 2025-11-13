use std::{collections::HashMap, rc::Rc};

mod inputs;
mod leaf_types;
mod normalisation;
mod sorting;
mod value;
mod variables;

use inputs::InputObjects;
use variables::VariableStructDetails;

pub use normalisation::{Directive, Variable};
pub use value::{LiteralContext, TypedValue};

use cynic_parser::{ExecutableDocument, executable as parser};

use crate::{
    Error, TypeIndex,
    casings::CasingExt,
    naming::Namer,
    output::{self, Output},
};

pub fn parse_query_document<'a>(
    doc: &'a ExecutableDocument,
    type_index: &Rc<TypeIndex<'a>>,
    field_overrides: &HashMap<&str, &str>
) -> Result<Output<'a, 'a>, Error> {
    let normalised = normalisation::normalise(doc, type_index)?;
    let input_objects = InputObjects::new(&normalised);

    let (mut enums, mut scalars) = leaf_types::extract_leaf_types(&normalised, &input_objects)?;

    enums.sort_by_key(|e| e.name);
    scalars.sort_by_key(|s| s.name);

    let mut namers = Namers::new();

    let variable_struct_details = variables::build_variable_structs(&normalised);

    for operation in &normalised.operations {
        let operation_name = operation.name.clone().unwrap_or("UnnamedQuery".into());

        namers
            .selection_sets
            .force_name(&operation.root, operation_name.clone());

        variable_struct_details
            .force_name_variables_for(&operation.root, format!("{operation_name}Variables"));
    }

    let query_fragments = sorting::topological_sort(normalised.selection_sets.iter().cloned())
        .into_iter()
        .map(|selection| make_query_fragment(selection, &mut namers, &variable_struct_details, field_overrides))
        .collect::<Vec<_>>();

    let inline_fragments = normalised
        .inline_fragments
        .into_iter()
        .map(|fragment| make_inline_fragments(fragment, &mut namers, &variable_struct_details))
        .collect::<Vec<_>>();

    let input_objects = input_objects.processed_objects();

    let enums = enums
        .into_iter()
        .map(|en| output::Enum {
            details: en,
            schema_name: None,
        })
        .collect();

    Ok(Output {
        query_fragments,
        inline_fragments,
        input_objects,
        enums,
        scalars,
        variables_structs: variable_struct_details.variables_structs(),
    })
}

fn make_query_fragment<'text>(
    selection: Rc<normalisation::SelectionSet<'text, 'text>>,
    namers: &mut Namers<'text>,
    variable_struct_details: &VariableStructDetails<'text, 'text>,
    field_overrides: &HashMap<&str, &str>,
) -> crate::output::QueryFragment<'text, 'text> {
    use crate::output::query_fragment::{
        FieldArgument, OutputField, QueryFragment, RustOutputFieldType,
    };
    use normalisation::{Field, Selection};

    let fragment_name = namers.selection_sets.name_subject(&selection);

    QueryFragment {
        fields: selection
            .selections
            .iter()
            .map(|selection| {
                let Selection::Field(field) = selection;
                let schema_field = &field.schema_field;

                let field_name = field.alias.unwrap_or(schema_field.name);

                let type_name_override = match &field.field {
                    Field::Leaf => field_overrides.get(format!("{}.{}", fragment_name, field_name).as_str()).map(|o| o.to_string()),
                    Field::Composite(ss) => Some(namers.selection_sets.name_subject(ss)),
                    Field::InlineFragments(fragments) => {
                        Some(namers.inline_fragments.name_subject(fragments))
                    }
                };

                OutputField {
                    name: field_name,
                    rename: field.alias.map(|_| schema_field.name).or_else(|| {
                        (schema_field.name.to_snake_case().to_camel_case() != schema_field.name)
                            .then_some(schema_field.name)
                    }),
                    field_type: RustOutputFieldType::from_schema_type(
                        &schema_field.value_type,
                        type_name_override,
                    ),
                    arguments: field
                        .arguments
                        .iter()
                        .map(|argument| -> Result<FieldArgument, Error> {
                            Ok(FieldArgument::new(argument.name, argument.value.clone()))
                        })
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap(),
                    directives: field.directives.clone(),
                }
            })
            .collect(),
        variable_struct_name: variable_struct_details.variables_name_for_selection(&selection),

        name: fragment_name,
        target_type: selection.target_type.name().to_string(),
        schema_name: None,
    }
}

fn make_inline_fragments<'text>(
    inline_fragments: Rc<normalisation::InlineFragments<'text, 'text>>,
    namers: &mut Namers<'text>,
    variable_struct_details: &VariableStructDetails<'text, 'text>,
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
        variable_struct_name: inline_fragments
            .inner_selections
            .iter()
            .filter_map(|selection| variable_struct_details.variables_name_for_selection(selection))
            .next(),
        name: namers.inline_fragments.name_subject(&inline_fragments),
        schema_name: None,
    }
}

#[derive(Debug)]
struct Namers<'text> {
    selection_sets: Namer<Rc<normalisation::SelectionSet<'text, 'text>>>,
    inline_fragments: Namer<Rc<normalisation::InlineFragments<'text, 'text>>>,
}

impl Namers<'_> {
    pub fn new() -> Self {
        Namers {
            selection_sets: Namer::new(),
            inline_fragments: Namer::new(),
        }
    }
}
