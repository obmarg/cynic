use std::{
    collections::{BTreeSet, HashMap},
    convert::TryInto,
    hash::Hash,
    rc::Rc,
};

use super::{
    parser::{
        self, Definition, Document, FragmentDefinition, OperationDefinition, TypeCondition,
        VariableDefinition,
    },
    sorting::Vertex,
    value::TypedValue,
};

use crate::{
    schema::{InputFieldType, InputTypeRef, OutputField, OutputType, OutputTypeRef, Type},
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
    Field(FieldSelection<'query, 'schema>),
    InlineFragment(SelectionSet<'query, 'schema>),
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
    ) -> FieldSelection<'query, 'schema> {
        FieldSelection {
            name,
            alias,
            arguments,
            schema_field,
            field,
        }
    }
}

// Use a BTreeSet here as we want a deterministic order of output for a
// given document
type SelectionSetSet<'query, 'schema> = BTreeSet<Rc<SelectionSet<'query, 'schema>>>;

#[derive(Debug, PartialEq)]
pub struct NormalisedDocument<'query, 'schema> {
    pub selection_sets: SelectionSetSet<'query, 'schema>,
    // TODO: also need to know what unions we've seen as part of this...
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
    fragment_map: &FragmentMap<'query, 'doc>,
    type_index: &'doc Rc<TypeIndex<'schema>>,
    selection_sets_out: &mut SelectionSetSet<'query, 'schema>,
) -> Result<NormalisedOperation<'query, 'schema>, Error> {
    match operation {
        OperationDefinition::SelectionSet(selection_set) => {
            let mut normaliser = Normaliser::new(type_index, fragment_map, selection_sets_out, &[]);
            let root =
                normaliser.normalise_selection_set(&selection_set, GraphPath::for_query())?;

            Ok(NormalisedOperation {
                root,
                name: None,
                kind: OperationKind::Query,
                variables: normaliser.variables,
            })
        }
        OperationDefinition::Query(query) => {
            let mut normaliser = Normaliser::new(
                type_index,
                fragment_map,
                selection_sets_out,
                &query.variable_definitions,
            );

            let root =
                normaliser.normalise_selection_set(&query.selection_set, GraphPath::for_query())?;

            Ok(NormalisedOperation {
                root,
                name: query.name,
                kind: OperationKind::Query,
                variables: normaliser.variables,
            })
        }
        OperationDefinition::Mutation(mutation) => {
            let mut normaliser = Normaliser::new(
                type_index,
                fragment_map,
                selection_sets_out,
                &mutation.variable_definitions,
            );

            let root = normaliser
                .normalise_selection_set(&mutation.selection_set, GraphPath::for_mutation())?;

            Ok(NormalisedOperation {
                root,
                name: mutation.name,
                kind: OperationKind::Mutation,
                variables: normaliser.variables,
            })
        }
        OperationDefinition::Subscription(_) => Err(Error::UnsupportedQueryDocument(
            "Subscriptions are not yet supported".into(),
        )),
    }
}

struct Normaliser<'a, 'query, 'schema, 'doc> {
    type_index: &'a Rc<TypeIndex<'schema>>,
    fragment_map: &'a FragmentMap<'query, 'doc>,
    selection_sets_out: &'a mut SelectionSetSet<'query, 'schema>,
    variables: Vec<Variable<'query, 'schema>>,
}

impl<'a, 'query, 'schema, 'doc> Normaliser<'a, 'query, 'schema, 'doc> {
    fn new(
        type_index: &'a Rc<TypeIndex<'schema>>,
        fragment_map: &'a FragmentMap<'query, 'doc>,
        selection_sets_out: &'a mut SelectionSetSet<'query, 'schema>,
        variable_definitions: &'a [parser::VariableDefinition<'query>],
    ) -> Self {
        Normaliser {
            type_index,
            fragment_map,
            selection_sets_out,
            variables: variable_definitions
                .iter()
                .map(|var| Variable::from(var, type_index))
                .collect(),
        }
    }

    fn normalise_selection_set(
        &mut self,
        selection_set: &parser::SelectionSet<'query>,
        current_path: GraphPath<'query>,
    ) -> Result<Rc<SelectionSet<'query, 'schema>>, Error> {
        let mut selections = Vec::new();

        for item in &selection_set.items {
            selections.extend(self.convert_selection(item, &current_path)?);
        }

        let rv = Rc::new(SelectionSet {
            target_type: self.type_index.type_for_path(&current_path)?.try_into()?,
            selections,
        });

        if let Some(existing_value) = self.selection_sets_out.get(&rv) {
            return Ok(Rc::clone(existing_value));
        }

        self.selection_sets_out.insert(Rc::clone(&rv));

        Ok(rv)
    }

    fn convert_selection(
        &mut self,
        selection: &parser::Selection<'query>,
        current_path: &GraphPath<'query>,
    ) -> Result<Vec<Selection<'query, 'schema>>, Error> {
        match selection {
            parser::Selection::Field(field) => {
                let new_path = current_path.push(field.name);

                let schema_field = self.type_index.field_for_path(&new_path)?;

                let inner_field = if field.selection_set.items.is_empty() {
                    if let OutputType::Object(_) = schema_field.value_type.inner_ref().lookup()? {
                        return Err(Error::NoFieldSelected(schema_field.name.into()));
                    } else {
                        Field::Leaf
                    }
                } else {
                    Field::Composite(self.normalise_selection_set(&field.selection_set, new_path)?)
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
                            &self.variables,
                        )?,
                    ));
                }

                Ok(vec![Selection::Field(FieldSelection::new(
                    field.name,
                    field.alias,
                    arguments,
                    schema_field,
                    inner_field,
                ))])
            }
            parser::Selection::FragmentSpread(spread) => {
                let fragment = self
                    .fragment_map
                    .get(spread.fragment_name)
                    .ok_or_else(|| Error::UnknownFragment(spread.fragment_name.to_string()))?;

                let TypeCondition::On(condition) = fragment.type_condition;
                let current_type = self.type_index.type_name_for_path(&current_path)?;
                if condition != current_type {
                    return Err(Error::TypeConditionFailed(
                        condition.to_string(),
                        current_type.to_string(),
                    ));
                }

                Ok(fragment
                    .selection_set
                    .items
                    .iter()
                    .map(|item| self.convert_selection(item, current_path))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .flatten()
                    .collect())
            }
            parser::Selection::InlineFragment(fragment) => {
                if let Some(TypeCondition::On(target_type_name)) = fragment.type_condition {
                    let current_type = self.type_index.type_for_path(&current_path)?;
                    let target_type = self.type_index.lookup_type(target_type_name)?;

                    // TODO: Probably want to handle the case where the target
                    // is still the current type.

                    current_type.allows_fragment_target_of(&target_type)?;

                    // Ok, so this is where it gets tricky.
                    // We need an enum with all the types that were provided by the user on
                    // the current branch.
                    // Basically we need the sibling selections.

                    // and a fallback.
                    // But there's no way to know which types were provided.
                    // Fucking lol god damn.
                    todo!()
                } else {
                    // If there's no type condition we assume this is a normal selection set on the current type
                    Ok(fragment
                        .selection_set
                        .items
                        .iter()
                        .map(|item| self.convert_selection(item, current_path))
                        .collect::<Result<Vec<_>, _>>()?
                        .into_iter()
                        .flatten()
                        .collect())
                }
            }
        }
    }
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
                if let Selection::Field(field) = selection {
                    if let Field::Leaf = &field.field {
                        return Some(field.schema_field.value_type.inner_ref().clone());
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
                    .map(|(_, arg)| arg.value_type().inner_ref().clone())
                    .collect(),
                Selection::InlineFragment(_) => vec![],
            })
            .collect()
    }
}

impl<'query, 'schema> crate::naming::Nameable for Rc<SelectionSet<'query, 'schema>> {
    fn requested_name(&self) -> String {
        self.target_type.name().to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema;

    use assert_matches::assert_matches;

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

    #[test]
    fn check_fragment_spread_output() {
        let schema = load_schema();
        let type_index = Rc::new(TypeIndex::from_schema(&schema));
        let query = graphql_parser::parse_query::<&str>(
            r#"
            fragment FilmFields on Film {
              id
              title
            }
            query AllFilms {
              allFilms {
                films {
                    ...FilmFields
                }
              }
              film(id: "abcd") {
                ...FilmFields
              }
            }
            "#,
        )
        .unwrap();

        let normalised = normalise(&query, &type_index).unwrap();

        let film_selections = normalised
            .selection_sets
            .iter()
            .filter(|s| s.target_type.name() == "Film")
            .collect::<Vec<_>>();

        assert_eq!(film_selections.len(), 1);

        insta::assert_debug_snapshot!(film_selections.get(0).unwrap().selections);
    }

    #[test]
    fn check_fragment_type_mismatches() {
        let schema = load_schema();
        let type_index = Rc::new(TypeIndex::from_schema(&schema));
        let query = graphql_parser::parse_query::<&str>(
            r#"
            fragment FilmFields on Film {
              id
              title
            }

            query AllFilms {
              allFilms {
                ...FilmFields
              }
            }
            "#,
        )
        .unwrap();

        assert_matches!(
            normalise(&query, &type_index),
            Err(Error::TypeConditionFailed(_, _))
        )
    }

    #[test]
    fn check_field_selected() {
        let schema = load_schema();
        let type_index = Rc::new(TypeIndex::from_schema(&schema));
        let query = graphql_parser::parse_query::<&str>(
            r#"
           query MyQuery {
              allFilms(after: "") {
                edges {
                  cursor
                  node {
                    created
                    edited
                    episodeID
                  }
                }
              }
            }
            "#,
        )
        .unwrap();
        let normalised = normalise(&query, &type_index).unwrap();

        let film_selections = normalised
            .selection_sets
            .iter()
            .map(|s| s.target_type.name());

        assert_eq!(film_selections.count(), 4);
    }

    #[test]
    fn check_no_field_selected() {
        let schema = load_schema();
        let type_index = Rc::new(TypeIndex::from_schema(&schema));
        let query = graphql_parser::parse_query::<&str>(
            r#"
           query MyQuery {
              allFilms(after: "") {
                edges {
                  cursor
                  node
                }
              }
            }
            "#,
        )
        .unwrap();
        assert_matches!(
            normalise(&query, &type_index),
            Err(Error::NoFieldSelected(_))
        )
    }

    #[test]
    fn check_inline_fragment_output() {
        let schema = load_schema();
        let type_index = Rc::new(TypeIndex::from_schema(&schema));
        let query = graphql_parser::parse_query::<&str>(
            r#"
            query AllFilms {
              allFilms {
                films {
                    ... on Film {
                      id
                    }
                    ... on Film {
                      title
                    }
                }
              }
            }
            "#,
        )
        .unwrap();

        let normalised = normalise(&query, &type_index).unwrap();

        let film_selections = normalised
            .selection_sets
            .iter()
            .filter(|s| s.target_type.name() == "Film")
            .collect::<Vec<_>>();

        assert_eq!(film_selections.len(), 1);

        insta::assert_debug_snapshot!(film_selections.get(0).unwrap().selections);
    }

    #[test]
    fn check_inline_fragment_type_mismatches() {
        let schema = load_schema();
        let type_index = Rc::new(TypeIndex::from_schema(&schema));
        let query = graphql_parser::parse_query::<&str>(
            r#"
            query AllFilms {
              allFilms {
                ... on Film {
                  id
                }
              }
            }
            "#,
        )
        .unwrap();

        assert_matches!(
            normalise(&query, &type_index),
            Err(Error::TypeConditionFailed(_, _))
        )
    }

    fn load_schema() -> schema::Document<'static> {
        graphql_parser::parse_schema::<&str>(include_str!(
            "../../../schemas/starwars.schema.graphql"
        ))
        .unwrap()
    }
}
