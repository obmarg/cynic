//! Generation of argument structs
use std::collections::{HashMap, HashSet};

use super::{
    normalisation::{Field, NormalisedDocument, Selection, SelectionSet, Variable},
    types::{ArgumentStruct, ArgumentStructField},
};
use crate::schema::InputFieldType;

pub fn build_argument_structs<'query, 'schema>(
    doc: &NormalisedDocument<'query, 'schema>,
) -> Vec<ArgumentStruct<'schema, 'query>> {
    let operation_argument_roots = doc
        .operations
        .iter()
        .map(|op| SelectionArguments::from_selection_set(&op.root))
        .collect::<Vec<_>>();

    let mut parent_map = HashMap::new();
    for argument_set in &operation_argument_roots {
        if let Some(argument_set) = argument_set {
            argument_set.build_parent_map(&mut parent_map);
        }
    }

    doc.operations
        .iter()
        .zip(&operation_argument_roots)
        .map(|(op, args)| Some(op).zip(args.as_ref()))
        .flatten()
        .map(|(op, args)| {
            let mut argument_struct = args.as_argument_struct(&parent_map);

            if let Some(op_name) = op.name {
                argument_struct.name = format!("{}Arguments", op_name);
            }

            // TODO: Get the nested argument_structs out _and_ implement conversions

            // TODO: ideally need to consolidate types based on the query definition.

            argument_struct
        })
        .collect()
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SelectionArguments<'query, 'schema> {
    target_type_name: &'schema str,
    fields: Vec<SelectionArgument<'query, 'schema>>,
}

impl<'query, 'schema> SelectionArguments<'query, 'schema> {
    fn from_selection_set(selection_set: &SelectionSet<'query, 'schema>) -> Option<Self> {
        let mut fields = Vec::new();
        for selection in &selection_set.selections {
            let Selection::Field(field) = selection;
            for (_, value) in &field.arguments {
                for variable in value.variables() {
                    fields.push(SelectionArgument::VariableArgument(variable));
                }
            }

            if let Field::Composite(inner_select) = &field.field {
                if let Some(sub_struct) = SelectionArguments::from_selection_set(&inner_select) {
                    fields.push(SelectionArgument::NestedArguments(sub_struct));
                }
            }
        }

        if fields.is_empty() {
            return None;
        }

        Some(SelectionArguments {
            target_type_name: selection_set.target_type.name(),
            fields,
        })
    }

    fn as_argument_struct(
        &self,
        parent_map: &HashMap<&Self, HashSet<&Self>>,
    ) -> ArgumentStruct<'schema, 'query> {
        // TODO: Finish this - need a way to output the full set of ArgumentStructs.
        // Possibly also need to name them...
        let fields = self
            .fields
            .iter()
            .flat_map(|field| match field {
                SelectionArgument::VariableArgument(var) => {
                    vec![ArgumentStructField::Variable(var.clone())]
                }
                SelectionArgument::NestedArguments(nested) => {
                    let parent_map_entry = parent_map.get(&nested);

                    let nested_struct = nested.as_argument_struct(parent_map);

                    if parent_map_entry.map(|hs| hs.len()).unwrap_or(0) <= 1 {
                        // This particular childs arguments are only used by it,
                        // so we can safely lift them up into our argument struct
                        nested_struct.fields
                    } else {
                        vec![ArgumentStructField::NestedStruct(nested_struct)]
                    }
                }
            })
            .collect();

        ArgumentStruct {
            name: format!("{}Arguments", self.target_type_name),
            fields,
        }
    }

    /// Builds up an adjacenecy hashmap of child -> parent
    fn build_parent_map<'a>(&'a self, parent_map: &mut HashMap<&'a Self, HashSet<&'a Self>>) {
        for field in &self.fields {
            if let SelectionArgument::NestedArguments(child) = field {
                parent_map
                    .entry(child)
                    .or_insert_with(HashSet::new)
                    .insert(self.clone());
            }
        }
    }

    fn nested_arguments(&self) -> Vec<&SelectionArguments<'query, 'schema>> {
        let mut rv = Vec::new();
        rv.push(self);
        for field in &self.fields {
            if let SelectionArgument::NestedArguments(nested) = field {
                rv.extend(nested.nested_arguments());
            }
        }

        rv
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SelectionArgument<'query, 'schema> {
    VariableArgument(Variable<'query, 'schema>),
    NestedArguments(SelectionArguments<'query, 'schema>),
}
