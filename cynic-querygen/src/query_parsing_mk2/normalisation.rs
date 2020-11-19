use std::{
    collections::{BTreeSet, HashMap},
    convert::TryInto,
    hash::Hash,
    rc::Rc,
};

use super::{sorting::Vertex, value::TypedValue};
use crate::{
    query::{
        self, Definition, Document, FragmentDefinition, OperationDefinition, VariableDefinition,
    },
    schema::{InputFieldType, InputTypeRef, OutputField, OutputType, OutputTypeRef},
    Error, GraphPath, TypeIndex,
};

#[derive(Debug, PartialEq)]
pub struct NormalisedOperation<'query, 'schema> {
    pub root: Rc<SelectionSet<'query, 'schema>>,
    pub name: Option<&'query str>,
    pub variables: Vec<Variable<'query, 'schema>>,
    pub kind: OperationKind,
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Hash)]
pub struct Variable<'query, 'schema> {
    pub name: &'query str,
    pub value_type: InputFieldType<'schema>,
}

#[derive(Debug, PartialEq)]
pub enum OperationKind {
    Query,
    Mutation,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SelectionSet<'query, 'schema> {
    pub target_type: OutputType<'schema>,
    pub selections: Vec<Selection<'query, 'schema>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Selection<'query, 'schema> {
    // For now I just care about fields
    // Will probably need InlineFragments here sometime
    // Figure a normal FragmentSpread can be normalised in place.
    Field(FieldSelection<'query, 'schema>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldSelection<'query, 'schema> {
    pub alias: Option<&'query str>,
    pub name: &'query str,

    pub schema_field: OutputField<'schema>,

    pub arguments: Vec<(&'schema str, TypedValue<'query, 'schema>)>,

    pub field: Field<'query, 'schema>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Field<'query, 'schema> {
    /// A composite field contains another selection set.
    Composite(Rc<SelectionSet<'query, 'schema>>),

    /// A leaf field just contains it's type as a string
    Leaf,
}

impl<'query, 'doc, 'schema> FieldSelection<'query, 'schema> {
    fn new(
        name: &'query str,
        alias: Option<&'query str>,
        arguments: Vec<(&'schema str, TypedValue<'query, 'schema>)>,
        schema_field: OutputField<'schema>,
        field: Field<'query, 'schema>,
    ) -> Result<FieldSelection<'query, 'schema>, Error> {
        Ok(FieldSelection {
            name,
            alias,
            arguments,
            schema_field,
            field,
        })
    }
}

/// Use a BTreeSet here as we want a deterministic order of output for a
/// given document
type SelectionSetSet<'query, 'schema> = BTreeSet<Rc<SelectionSet<'query, 'schema>>>;

#[derive(Debug, PartialEq)]
pub struct NormalisedDocument<'query, 'schema> {
    pub selection_sets: SelectionSetSet<'query, 'schema>,
    pub operations: Vec<NormalisedOperation<'query, 'schema>>,
}

pub fn normalise<'query, 'doc, 'schema>(
    document: &'doc Document<'query>,
    type_index: &'doc Rc<TypeIndex<'schema>>,
) -> Result<NormalisedDocument<'query, 'schema>, Error> {
    let fragment_map = extract_fragments(&document);

    let mut selection_sets: SelectionSetSet<'query, 'schema> = BTreeSet::new();
    let mut operations = Vec::new();

    for definition in &document.definitions {
        if let Definition::Operation(operation) = definition {
            operations.push(normalise_operation(
                operation,
                &fragment_map,
                type_index,
                &mut selection_sets,
            )?);
        }
    }

    Ok(NormalisedDocument {
        selection_sets,
        operations,
    })
}

fn normalise_operation<'query, 'doc, 'schema>(
    operation: &'doc OperationDefinition<'query>,
    _fragment_map: &FragmentMap<'query, 'doc>,
    type_index: &'doc Rc<TypeIndex<'schema>>,
    selection_sets_out: &mut SelectionSetSet<'query, 'schema>,
) -> Result<NormalisedOperation<'query, 'schema>, Error> {
    match operation {
        OperationDefinition::SelectionSet(selection_set) => {
            let root = normalise_selection_set(
                &selection_set,
                type_index,
                GraphPath::for_query(),
                &[],
                selection_sets_out,
            )?;

            Ok(NormalisedOperation {
                root,
                name: None,
                kind: OperationKind::Query,
                variables: vec![],
            })
        }
        OperationDefinition::Query(query) => {
            let variables = query
                .variable_definitions
                .iter()
                .map(|var| Variable::from(var, type_index))
                .collect::<Vec<_>>();

            let root = normalise_selection_set(
                &query.selection_set,
                type_index,
                GraphPath::for_query(),
                &variables,
                selection_sets_out,
            )?;

            Ok(NormalisedOperation {
                root,
                name: query.name,
                kind: OperationKind::Query,
                variables,
            })
        }
        OperationDefinition::Mutation(mutation) => {
            let variables = mutation
                .variable_definitions
                .iter()
                .map(|var| Variable::from(var, type_index))
                .collect::<Vec<_>>();

            let root = normalise_selection_set(
                &mutation.selection_set,
                type_index,
                GraphPath::for_mutation(),
                &variables,
                selection_sets_out,
            )?;

            Ok(NormalisedOperation {
                root,
                name: mutation.name,
                kind: OperationKind::Mutation,
                variables,
            })
        }
        OperationDefinition::Subscription(_) => Err(Error::UnsupportedQueryDocument(
            "Subscriptions are not yet supported".into(),
        )),
    }
}

fn normalise_selection_set<'query, 'schema>(
    selection_set: &query::SelectionSet<'query>,
    type_index: &Rc<TypeIndex<'schema>>,
    current_path: GraphPath<'query>,
    variable_definitions: &[Variable<'query, 'schema>],
    selection_sets_out: &mut SelectionSetSet<'query, 'schema>,
) -> Result<Rc<SelectionSet<'query, 'schema>>, Error> {
    let mut selections = Vec::new();

    for item in &selection_set.items {
        match item {
            query::Selection::Field(field) => {
                let new_path = current_path.push(field.name);

                let schema_field = type_index.field_for_path_2(&new_path)?;

                let inner_field = if field.selection_set.items.is_empty() {
                    Field::Leaf
                } else {
                    Field::Composite(normalise_selection_set(
                        &field.selection_set,
                        type_index,
                        new_path,
                        variable_definitions,
                        selection_sets_out,
                    )?)
                };

                let mut arguments = Vec::new();
                for (name, value) in &field.arguments {
                    let schema_arg = schema_field
                        .arguments
                        .iter()
                        .find(|arg| arg.name == *name)
                        .ok_or_else(|| Error::UnknownArgument(name.to_string()))?;

                    arguments.push((
                        schema_arg.name,
                        TypedValue::from_query_value(
                            value,
                            schema_arg.value_type.clone(),
                            variable_definitions,
                        )?,
                    ));
                }

                selections.push(Selection::Field(FieldSelection::new(
                    field.name,
                    field.alias,
                    arguments,
                    schema_field,
                    inner_field,
                )?));
            }
            query::Selection::FragmentSpread(_) => todo!(),
            query::Selection::InlineFragment(_) => todo!(),
        }
    }

    let rv = Rc::new(SelectionSet {
        target_type: type_index.type_for_path(&current_path)?.try_into()?,
        selections,
    });

    if let Some(existing_value) = selection_sets_out.get(&rv) {
        return Ok(Rc::clone(existing_value));
    }

    selection_sets_out.insert(Rc::clone(&rv));

    Ok(rv)
}

impl<'query, 'schema> Variable<'query, 'schema> {
    fn from(def: &VariableDefinition<'query>, type_index: &Rc<TypeIndex<'schema>>) -> Self {
        Variable {
            name: def.name,
            value_type: InputFieldType::from_variable_definition(def, type_index),
        }
    }
}

type FragmentMap<'query, 'doc> = HashMap<&'query str, &'doc FragmentDefinition<'query>>;

fn extract_fragments<'query, 'doc>(document: &'doc Document<'query>) -> FragmentMap<'query, 'doc> {
    document
        .definitions
        .iter()
        .flat_map(|definition| {
            if let Definition::Fragment(fragment) = definition {
                Some((fragment.name, fragment))
            } else {
                None
            }
        })
        .collect()
}

impl<'query, 'schema> Vertex for SelectionSet<'query, 'schema> {
    fn adjacents(self: &Rc<Self>) -> Vec<Rc<Self>> {
        self.selections
            .iter()
            .flat_map(|selection| match selection {
                Selection::Field(FieldSelection {
                    field: Field::Composite(selection_set),
                    ..
                }) => Some(Rc::clone(&selection_set)),
                _ => None,
            })
            .collect()
    }
}

impl<'query, 'schema> SelectionSet<'query, 'schema> {
    pub fn leaf_output_types(&self) -> Vec<OutputTypeRef<'schema>> {
        self.selections
            .iter()
            .flat_map(|selection| {
                match selection {
                    Selection::Field(field) => {
                        if let Field::Leaf = &field.field {
                            return Some(field.schema_field.value_type.inner_ref().clone());
                        }
                    }
                }
                None
            })
            .collect()
    }

    pub fn required_input_types(&self) -> Vec<InputTypeRef<'schema>> {
        self.selections
            .iter()
            .flat_map(|selection| match selection {
                Selection::Field(sel) => sel
                    .arguments
                    .iter()
                    .map(|(_, arg)| arg.value_type().inner_ref().clone()),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema;

    #[test]
    fn normalise_deduplicates_identical_selections() {
        let schema = load_schema();
        let type_index = Rc::new(TypeIndex::from_schema(&schema));
        let query = graphql_parser::parse_query::<&str>(
            r#"
            {
              allFilms {
                films {
                  id
                  title
                }
              }
              film(id: "abcd") {
                id
                title
              }
            }
            "#,
        )
        .unwrap();

        let normalised = normalise(&query, &type_index).unwrap();

        assert_eq!(
            normalised
                .selection_sets
                .iter()
                .filter(|s| s.target_type.name() == "Film")
                .count(),
            1
        );
    }

    #[test]
    fn normalise_does_not_deduplicate_differing_selections() {
        let schema = load_schema();
        let type_index = Rc::new(TypeIndex::from_schema(&schema));
        let query = graphql_parser::parse_query::<&str>(
            r#"
            {
              allFilms {
                films {
                  id
                  title
                }
              }
              film(id: "abcd") {
                title
              }
            }
            "#,
        )
        .unwrap();

        let normalised = normalise(&query, &type_index).unwrap();

        assert_eq!(
            normalised
                .selection_sets
                .iter()
                .filter(|s| s.target_type.name() == "Film")
                .count(),
            2
        );
    }

    #[test]
    fn check_output_makes_sense() {
        let schema = load_schema();
        let type_index = Rc::new(TypeIndex::from_schema(&schema));
        let query = graphql_parser::parse_query::<&str>(
            r#"
            {
              allFilms {
                films {
                  id
                  title
                }
              }
              film(id: "abcd") {
                title
              }
            }
            "#,
        )
        .unwrap();

        let normalised = normalise(&query, &type_index).unwrap();

        insta::assert_debug_snapshot!(normalised);
    }

    fn load_schema() -> schema::Document<'static> {
        graphql_parser::parse_schema::<&str>(include_str!(
            "../../../examples/examples/starwars.schema.graphql"
        ))
        .unwrap()
    }
}
