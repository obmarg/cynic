use std::collections::{BTreeSet, HashMap};
use std::hash::Hash;
use std::rc::Rc;

use super::{sorting::Vertex, value::Value};
use crate::{
    query::{
        self, Definition, Document, FragmentDefinition, OperationDefinition, VariableDefinition,
    },
    Error, GraphPath, TypeIndex,
};

#[derive(Debug, PartialEq)]
pub struct NormalisedOperation<'query> {
    root: Rc<SelectionSet<'query>>,
    pub name: Option<&'query str>,
    pub variable_definitions: Vec<VariableDefinition<'query>>,
    pub kind: OperationKind,
}

#[derive(Debug, PartialEq)]
pub enum OperationKind {
    Query,
    Mutation,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SelectionSet<'query> {
    pub target_type: String,
    pub selections: Vec<Selection<'query>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Selection<'query> {
    // For now I just care about fields
    // Will probably need InlineFragments here sometime
    // Figure a normal FragmentSpread can be normalised in place.
    Field(FieldSelection<'query>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldSelection<'query> {
    pub alias: Option<&'query str>,
    pub name: &'query str,
    pub arguments: Vec<(&'query str, Value<'query>)>,
    pub field: Field<'query>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Field<'query> {
    /// A composite field contains another selection set.
    Composite(Rc<SelectionSet<'query>>),

    /// A leaf field just contains it's type as a string
    Leaf(String),
}

impl<'query, 'doc> FieldSelection<'query> {
    fn new(
        name: &'query str,
        alias: Option<&'query str>,
        arguments: &'doc [(&'query str, query::Value<'query>)],
        field: Field<'query>,
    ) -> FieldSelection<'query> {
        let arguments = arguments
            .iter()
            .map(|(k, v)| (*k, Value::from(v)))
            .collect::<Vec<_>>();

        FieldSelection {
            name,
            alias,
            arguments,
            field,
        }
    }
}

/// Use a BTreeSet here as we want a deterministic order of output for a
/// given document
type SelectionSetSet<'query> = BTreeSet<Rc<SelectionSet<'query>>>;

#[derive(Debug, PartialEq)]
pub struct NormalisedDocument<'query> {
    pub selection_sets: SelectionSetSet<'query>,
    pub operations: Vec<NormalisedOperation<'query>>,
}

pub fn normalise<'query, 'doc>(
    document: &'doc Document<'query>,
    type_index: &'doc TypeIndex<'query>,
) -> Result<NormalisedDocument<'query>, Error> {
    let fragment_map = extract_fragments(&document);

    let mut selection_sets: SelectionSetSet<'query> = BTreeSet::new();
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

fn normalise_operation<'query, 'doc>(
    operation: &'doc OperationDefinition<'query>,
    fragment_map: &FragmentMap<'query, 'doc>,
    type_index: &'doc TypeIndex<'query>,
    selection_sets_out: &mut SelectionSetSet<'query>,
) -> Result<NormalisedOperation<'query>, Error> {
    match operation {
        OperationDefinition::SelectionSet(selection_set) => {
            let root = normalise_selection_set(
                &selection_set,
                type_index,
                GraphPath::for_query(),
                selection_sets_out,
            )?;

            Ok(NormalisedOperation {
                root,
                name: None,
                kind: OperationKind::Query,
                variable_definitions: vec![],
            })
        }
        OperationDefinition::Query(query) => {
            let root = normalise_selection_set(
                &query.selection_set,
                type_index,
                GraphPath::for_query(),
                selection_sets_out,
            )?;

            Ok(NormalisedOperation {
                root,
                name: query.name,
                kind: OperationKind::Query,
                variable_definitions: query.variable_definitions.iter().cloned().collect(),
            })
        }
        OperationDefinition::Mutation(mutation) => {
            let root = normalise_selection_set(
                &mutation.selection_set,
                type_index,
                GraphPath::for_mutation(),
                selection_sets_out,
            )?;

            Ok(NormalisedOperation {
                root,
                name: mutation.name,
                kind: OperationKind::Mutation,
                variable_definitions: mutation.variable_definitions.iter().cloned().collect(),
            })
        }
        OperationDefinition::Subscription(_) => Err(Error::UnsupportedQueryDocument(
            "Subscriptions are not yet supported".into(),
        )),
    }
}

fn normalise_selection_set<'query, 'doc>(
    selection_set: &'doc query::SelectionSet<'query>,
    type_index: &'doc TypeIndex<'query>,
    current_path: GraphPath<'query>,
    selection_sets_out: &mut SelectionSetSet<'query>,
) -> Result<Rc<SelectionSet<'query>>, Error> {
    use crate::type_ext::TypeExt;

    let mut selections = Vec::new();

    for item in &selection_set.items {
        match item {
            query::Selection::Field(field) => {
                let new_path = current_path.push(field.name);

                let inner_field = if field.selection_set.items.is_empty() {
                    Field::Leaf(
                        type_index
                            .field_for_path(&new_path)?
                            .field_type
                            .inner_name()
                            .to_string(),
                    )
                } else {
                    Field::Composite(normalise_selection_set(
                        &field.selection_set,
                        type_index,
                        new_path,
                        selection_sets_out,
                    )?)
                };

                selections.push(Selection::Field(FieldSelection::new(
                    field.name,
                    field.alias,
                    &field.arguments,
                    inner_field,
                )));
            }
            query::Selection::FragmentSpread(_) => todo!(),
            query::Selection::InlineFragment(_) => todo!(),
        }
    }

    let rv = Rc::new(SelectionSet {
        target_type: type_index.type_name_for_path(&current_path)?.to_string(),
        selections,
    });

    if let Some(existing_value) = selection_sets_out.get(&rv) {
        return Ok(Rc::clone(existing_value));
    }

    selection_sets_out.insert(Rc::clone(&rv));

    Ok(rv)
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

impl<'query> Vertex for SelectionSet<'query> {
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

impl<'query> SelectionSet<'query> {
    pub fn leaf_type_names<'a>(&'a self) -> impl Iterator<Item = &'a str> {
        self.selections
            .iter()
            .flat_map(|selection| match selection {
                Selection::Field(field) => {
                    let mut rv = Vec::new();
                    if let Field::Leaf(named_type) = &field.field {
                        rv.push(named_type.as_ref());
                    }
                    rv
                }
                _ => vec![],
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema;

    #[test]
    fn normalise_deduplicates_identical_selections() {
        let schema = load_schema();
        let type_index = TypeIndex::from_schema(&schema);
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
                .filter(|s| s.target_type == "Film")
                .count(),
            1
        );
    }

    #[test]
    fn normalise_does_not_deduplicate_differing_selections() {
        let schema = load_schema();
        let type_index = TypeIndex::from_schema(&schema);
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
                .filter(|s| s.target_type == "Film")
                .count(),
            2
        );
    }

    #[test]
    fn check_output_makes_sense() {
        let schema = load_schema();
        let type_index = TypeIndex::from_schema(&schema);
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
