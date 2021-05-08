//! Generation of argument structs
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use uuid::Uuid;

use super::normalisation::{Field, NormalisedDocument, Selection, SelectionSet, Variable};
use crate::{naming::Namer, output};

pub fn build_argument_structs<'query, 'schema, 'doc>(
    doc: &'doc NormalisedDocument<'query, 'schema>,
) -> ArgumentStructDetails<'query, 'schema, 'doc> {
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

    let mut output = ArgumentStructDetails::new();

    for args in &operation_argument_roots {
        if let Some(args) = args {
            args.as_argument_struct(&parent_map, &mut output);
        }
        // TODO: implement conversions on nested types?
    }

    output
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SelectionArguments<'query, 'schema, 'doc> {
    target_selection: &'doc SelectionSet<'query, 'schema>,
    fields: Vec<SelectionArgument<'query, 'schema, 'doc>>,
}

impl<'query, 'schema, 'doc> SelectionArguments<'query, 'schema, 'doc> {
    fn from_selection_set(selection_set: &'doc SelectionSet<'query, 'schema>) -> Option<Self> {
        let mut fields = Vec::new();
        for selection in &selection_set.selections {
            match selection {
                Selection::Field(field) => {
                    for (_, value) in &field.arguments {
                        for variable in value.variables() {
                            fields.push(SelectionArgument::VariableArgument(variable));
                        }
                    }

                    if let Field::Composite(inner_select) = &field.field {
                        if let Some(sub_struct) =
                            SelectionArguments::from_selection_set(&inner_select)
                        {
                            fields.push(SelectionArgument::NestedArguments(sub_struct));
                        }
                    }
                }
                Selection::InlineFragment(inner) => {
                    if let Some(inner_args) = SelectionArguments::from_selection_set(inner) {
                        fields.push(SelectionArgument::NestedArguments(inner_args))
                    }
                }
            }
        }

        if fields.is_empty() {
            return None;
        }

        Some(SelectionArguments {
            target_selection: selection_set,
            fields,
        })
    }

    fn as_argument_struct(
        &self,
        parent_map: &HashMap<&Self, HashSet<&Self>>,
        output_mapping: &mut ArgumentStructDetails<'query, 'schema, 'doc>,
    ) -> Rc<ArgumentStruct<'query, 'schema>> {
        let our_id = Uuid::new_v4();

        let fields = self
            .fields
            .iter()
            .flat_map(|field| match field {
                SelectionArgument::VariableArgument(var) => {
                    vec![ArgumentStructField::Variable(var.clone())]
                }
                SelectionArgument::NestedArguments(nested) => {
                    let nested_struct = nested.as_argument_struct(parent_map, output_mapping);

                    if parent_map.get(&nested).map(|hs| hs.len()).unwrap_or(0) <= 1 {
                        // This particular childs arguments are only used by it,
                        // so we can safely lift them up into our argument struct
                        output_mapping.selection_structs.remove(&nested_struct.id);
                        output_mapping.remappings.insert(nested_struct.id, our_id);

                        Rc::try_unwrap(nested_struct).unwrap().fields
                    } else {
                        vec![ArgumentStructField::NestedStruct(nested_struct)]
                    }
                }
            })
            .collect();

        let rv = Rc::new(ArgumentStruct {
            id: our_id,
            target_type_name: self.target_selection.target_type.name().to_string(),
            fields,
        });

        output_mapping
            .selection_set_map
            .insert(self.target_selection, rv.id);
        output_mapping
            .selection_structs
            .insert(our_id, Rc::clone(&rv));

        rv
    }

    /// Builds up an adjacenecy hashmap of child -> parent
    fn build_parent_map<'a>(&'a self, parent_map: &mut HashMap<&'a Self, HashSet<&'a Self>>) {
        for field in &self.fields {
            if let SelectionArgument::NestedArguments(child) = field {
                parent_map
                    .entry(child)
                    .or_insert_with(HashSet::new)
                    .insert(self);
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ArgumentStruct<'query, 'schema> {
    id: Uuid,
    target_type_name: String,
    fields: Vec<ArgumentStructField<'query, 'schema>>,
}

impl<'query, 'schema> crate::naming::Nameable for Rc<ArgumentStruct<'query, 'schema>> {
    fn requested_name(&self) -> String {
        format!("{}Arguments", self.target_type_name)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ArgumentStructField<'query, 'schema> {
    Variable(Variable<'query, 'schema>),
    NestedStruct(Rc<ArgumentStruct<'query, 'schema>>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SelectionArgument<'query, 'schema, 'doc> {
    VariableArgument(Variable<'query, 'schema>),
    NestedArguments(SelectionArguments<'query, 'schema, 'doc>),
}

/// Keeps track of what selection sets use which argument structs
#[derive(Debug)]
pub struct ArgumentStructDetails<'query, 'schema, 'doc> {
    selection_set_map: HashMap<&'doc SelectionSet<'query, 'schema>, Uuid>,
    remappings: HashMap<Uuid, Uuid>,
    selection_structs: HashMap<Uuid, Rc<ArgumentStruct<'query, 'schema>>>,
    namer: RefCell<Namer<Rc<ArgumentStruct<'query, 'schema>>>>,
}

impl<'query, 'schema, 'doc> ArgumentStructDetails<'query, 'schema, 'doc> {
    fn new() -> Self {
        ArgumentStructDetails {
            selection_set_map: HashMap::new(),
            remappings: HashMap::new(),
            selection_structs: HashMap::new(),
            namer: RefCell::new(Namer::new()),
        }
    }

    pub fn argument_structs(self) -> Vec<output::ArgumentStruct<'query, 'schema>> {
        self.selection_structs
            .iter()
            .map(|(_, arg_struct)| {
                let name = self.namer.borrow_mut().name_subject(&arg_struct);
                output::ArgumentStruct::new(
                    name,
                    arg_struct
                        .fields
                        .iter()
                        .map(|field| match field {
                            ArgumentStructField::Variable(var) => {
                                output::ArgumentStructField::Variable(var.clone())
                            }
                            ArgumentStructField::NestedStruct(nested_struct) => {
                                let nested_name =
                                    self.namer.borrow_mut().name_subject(nested_struct);

                                output::ArgumentStructField::NestedStruct(nested_name)
                            }
                        })
                        .collect(),
                )
            })
            .collect()
    }

    fn lookup_args_for_selection(
        &self,
        selection_set: &SelectionSet<'query, 'schema>,
    ) -> Option<&Rc<ArgumentStruct<'query, 'schema>>> {
        self.selection_set_map
            .get(selection_set)
            .and_then(|mut id| {
                while let Some(remapped_id) = self.remappings.get(id) {
                    id = remapped_id;
                }
                self.selection_structs.get(id)
            })
    }

    pub fn argument_name_for_selection(
        &self,
        selection_set: &SelectionSet<'query, 'schema>,
    ) -> Option<String> {
        self.lookup_args_for_selection(selection_set)
            .map(|arg_struct| self.namer.borrow_mut().name_subject(arg_struct))
    }

    pub fn force_name_argument_struct_for(
        &self,
        selection_set: &SelectionSet<'query, 'schema>,
        name: String,
    ) {
        self.lookup_args_for_selection(selection_set)
            .map(|arg_struct| self.namer.borrow_mut().force_name(arg_struct, name));
    }
}
