use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};

use crate::{
    query::{
        self, Definition, Document, FragmentDefinition, OperationDefinition, VariableDefinition,
    },
    Error, GraphPath, TypeIndex,
};

#[derive(Debug, PartialEq)]
struct NormalisedOperation<'query, 'doc> {
    root: Rc<SelectionSet<'query, 'doc>>,
    name: Option<&'query str>,
    variable_definitions: Vec<&'doc VariableDefinition<'query>>,
    kind: OperationKind,
}

#[derive(Debug, PartialEq)]
enum OperationKind {
    Query,
    Mutation,
}

#[derive(Hash, PartialEq, Eq, Debug)]
struct SelectionSet<'query, 'doc> {
    target_type: String,
    selections: Vec<Selection<'query, 'doc>>,
}

#[derive(Hash, PartialEq, Eq, Debug)]
enum Selection<'query, 'doc> {
    // For now I just care about fields
    // Will probably need InlineFragments here sometime
    // Figure a normal FragmentSpread can be normalised in place.
    Field(FieldSelection<'query, 'doc>),
}

#[derive(Debug)]
struct FieldSelection<'query, 'doc> {
    alias: Option<&'query str>,
    name: &'query str,
    arguments: Vec<(&'query str, HashableValue<'query, 'doc>)>,
    //  Problem here is we can't just store Value as that isn't hashable...
    //  So either some hashable wrapper type
    //  or a full translation (probably the former)
    selection_set: Weak<SelectionSet<'query, 'doc>>,

    // Weak is not hashable so we need to take a hash when we create
    // the FieldSelection
    hash: u64,
}

impl<'query, 'doc> FieldSelection<'query, 'doc> {
    fn new(
        name: &'query str,
        alias: Option<&'query str>,
        arguments: &'doc [(&'query str, query::Value<'query>)],
        selection_set: &Rc<SelectionSet<'query, 'doc>>,
    ) -> FieldSelection<'query, 'doc> {
        let arguments = arguments
            .iter()
            .map(|(k, v)| (*k, HashableValue::new(v)))
            .collect::<Vec<_>>();

        let mut hasher = std::collections::hash_map::DefaultHasher::new();

        name.hash(&mut hasher);
        alias.hash(&mut hasher);
        arguments.hash(&mut hasher);
        selection_set.hash(&mut hasher);

        let hash = hasher.finish();

        let selection_set = Rc::downgrade(&selection_set);

        FieldSelection {
            name,
            alias,
            arguments,
            selection_set,
            hash,
        }
    }
}

impl<'query, 'doc> Hash for FieldSelection<'query, 'doc> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl<'query, 'doc> PartialEq for FieldSelection<'query, 'doc> {
    fn eq(&self, other: &FieldSelection) -> bool {
        // TODO: Should probably implement an actual equals here...
        self.hash == other.hash
    }
}

impl<'query, 'doc> Eq for FieldSelection<'query, 'doc> {}

#[derive(PartialEq, Debug)]
struct HashableValue<'query, 'doc> {
    inner: &'doc query::Value<'query>,
}

impl<'query, 'doc> HashableValue<'query, 'doc> {
    fn new(inner: &'doc query::Value<'query>) -> Self {
        HashableValue { inner }
    }
}

// Note: Technically this is wrong - a HashableValue _could_
// contain a floating point which is not Eq.
//
// But in practice I hope we'll be OK
impl<'query, 'doc> Eq for HashableValue<'query, 'doc> {}

impl<'query, 'doc> Hash for HashableValue<'query, 'doc> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use query::Value;

        match self.inner {
            Value::Variable(var) => var.hash(state),
            Value::Int(num) => num.as_i64().hash(state),
            Value::Float(num) => {
                // Can't hash an f64, so convert it to a string.
                // Feel that there are definitely edge cases to this
                // but not sure how else to approach it right now.
                num.to_string().hash(state);
            }
            Value::String(s) => s.hash(state),
            Value::Boolean(b) => b.hash(state),
            Value::Null => ().hash(state),
            Value::Enum(s) => s.hash(state),
            Value::List(values) => {
                for value in values {
                    HashableValue::new(value).hash(state);
                }
            }
            Value::Object(obj) => {
                for (k, v) in obj {
                    k.hash(state);
                    HashableValue::new(v).hash(state)
                }
            }
        }
    }
}

type SelectionSetSet<'query, 'doc> = HashSet<Rc<SelectionSet<'query, 'doc>>>;

#[derive(Debug, PartialEq)]
struct NormalisedDocument<'query, 'doc> {
    selection_sets: SelectionSetSet<'query, 'doc>,
    operations: Vec<NormalisedOperation<'query, 'doc>>,
}

// TODO: Make this (and all the types) public
fn normalise<'query, 'doc>(
    document: &'doc Document<'query>,
    type_index: &'doc TypeIndex<'query>,
) -> Result<NormalisedDocument<'query, 'doc>, Error> {
    let fragment_map = extract_fragments(&document);

    let mut selection_sets: SelectionSetSet<'query, 'doc> = HashSet::new();
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
    selection_sets_out: &mut SelectionSetSet<'query, 'doc>,
) -> Result<NormalisedOperation<'query, 'doc>, Error> {
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
                variable_definitions: query.variable_definitions.iter().collect(),
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
                variable_definitions: mutation.variable_definitions.iter().collect(),
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
    selection_sets_out: &mut SelectionSetSet<'query, 'doc>,
) -> Result<Rc<SelectionSet<'query, 'doc>>, Error> {
    let mut selections = Vec::new();

    for item in &selection_set.items {
        match item {
            query::Selection::Field(field) => {
                let new_path = current_path.push(field.name);

                let normalised = normalise_selection_set(
                    &field.selection_set,
                    type_index,
                    new_path,
                    selection_sets_out,
                )?;

                selections.push(Selection::Field(FieldSelection::new(
                    field.name,
                    field.alias,
                    &field.arguments,
                    &normalised,
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
    fn are_there_any_other_test_i_want() {
        todo!()
    }

    fn load_schema() -> schema::Document<'static> {
        graphql_parser::parse_schema::<&str>(include_str!(
            "../../../examples/examples/starwars.schema.graphql"
        ))
        .unwrap()
    }
}
