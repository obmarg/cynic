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
    schema::{InputFieldType, InputTypeRef, OutputField, OutputType, OutputTypeRef, Type, TypeRef},
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
    Subscription,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SelectionSet<'query, 'schema> {
    pub target_type: OutputType<'schema>,
    pub selections: Vec<Selection<'query, 'schema>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Selection<'query, 'schema> {
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

    /// An inline fragments
    InlineFragments(Rc<InlineFragments<'query, 'schema>>),

    /// A leaf field just contains it's type as a string
    Leaf,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InlineFragments<'query, 'schema> {
    pub abstract_type: Type<'schema>,
    pub inner_selections: Vec<Rc<SelectionSet<'query, 'schema>>>,
}

impl<'query, 'schema> Field<'query, 'schema> {
    pub fn selection_sets(&self) -> Vec<Rc<SelectionSet<'query, 'schema>>> {
        match self {
            Field::Composite(selection_set) => vec![Rc::clone(selection_set)],
            Field::InlineFragments(fragments) => fragments.inner_selections.clone(),
            Field::Leaf => vec![],
        }
    }
}

impl<'query, 'schema> FieldSelection<'query, 'schema> {
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
pub type InlineFragmentsSet<'query, 'schema> = BTreeSet<Rc<InlineFragments<'query, 'schema>>>;

#[derive(Debug, PartialEq)]
pub struct NormalisedDocument<'query, 'schema> {
    pub selection_sets: SelectionSetSet<'query, 'schema>,
    pub inline_fragments: InlineFragmentsSet<'query, 'schema>,
    pub operations: Vec<NormalisedOperation<'query, 'schema>>,
}

pub fn normalise<'query, 'doc, 'schema>(
    document: &'doc Document<'query>,
    type_index: &'doc Rc<TypeIndex<'schema>>,
) -> Result<NormalisedDocument<'query, 'schema>, Error> {
    let fragment_map = extract_fragments(document);

    let mut selection_sets: SelectionSetSet<'query, 'schema> = BTreeSet::new();
    let mut inline_fragments: InlineFragmentsSet<'query, 'schema> = BTreeSet::new();
    let mut operations = Vec::new();

    for definition in &document.definitions {
        if let Definition::Operation(operation) = definition {
            operations.push(normalise_operation(
                operation,
                &fragment_map,
                type_index,
                &mut selection_sets,
                &mut inline_fragments,
            )?);
        }
    }

    Ok(NormalisedDocument {
        selection_sets,
        inline_fragments,
        operations,
    })
}

fn normalise_operation<'query, 'doc, 'schema>(
    operation: &'doc OperationDefinition<'query>,
    fragment_map: &FragmentMap<'query, 'doc>,
    type_index: &'doc Rc<TypeIndex<'schema>>,
    selection_sets_out: &mut SelectionSetSet<'query, 'schema>,
    inline_fragments_out: &mut InlineFragmentsSet<'query, 'schema>,
) -> Result<NormalisedOperation<'query, 'schema>, Error> {
    match operation {
        OperationDefinition::SelectionSet(selection_set) => {
            let mut normaliser = Normaliser::new(
                type_index,
                fragment_map,
                selection_sets_out,
                inline_fragments_out,
                &[],
            );
            let root =
                normaliser.normalise_object_selection_set(selection_set, GraphPath::for_query())?;

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
                inline_fragments_out,
                &query.variable_definitions,
            );

            let root = normaliser
                .normalise_object_selection_set(&query.selection_set, GraphPath::for_query())?;

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
                inline_fragments_out,
                &mutation.variable_definitions,
            );

            let root = normaliser.normalise_object_selection_set(
                &mutation.selection_set,
                GraphPath::for_mutation(),
            )?;

            Ok(NormalisedOperation {
                root,
                name: mutation.name,
                kind: OperationKind::Mutation,
                variables: normaliser.variables,
            })
        }
        OperationDefinition::Subscription(subscription) => {
            let mut normaliser = Normaliser::new(
                type_index,
                fragment_map,
                selection_sets_out,
                inline_fragments_out,
                &subscription.variable_definitions,
            );

            let root = normaliser.normalise_object_selection_set(
                &subscription.selection_set,
                GraphPath::for_subscription(),
            )?;

            Ok(NormalisedOperation {
                root,
                name: subscription.name,
                kind: OperationKind::Subscription,
                variables: normaliser.variables,
            })
        }
    }
}

struct Normaliser<'a, 'query, 'schema, 'doc> {
    type_index: &'a Rc<TypeIndex<'schema>>,
    fragment_map: &'a FragmentMap<'query, 'doc>,
    selection_sets_out: &'a mut SelectionSetSet<'query, 'schema>,
    inline_fragments_out: &'a mut InlineFragmentsSet<'query, 'schema>,
    variables: Vec<Variable<'query, 'schema>>,
}

impl<'a, 'query, 'schema, 'doc> Normaliser<'a, 'query, 'schema, 'doc> {
    fn new(
        type_index: &'a Rc<TypeIndex<'schema>>,
        fragment_map: &'a FragmentMap<'query, 'doc>,
        selection_sets_out: &'a mut SelectionSetSet<'query, 'schema>,
        inline_fragments_out: &'a mut InlineFragmentsSet<'query, 'schema>,
        variable_definitions: &'a [parser::VariableDefinition<'query>],
    ) -> Self {
        Normaliser {
            type_index,
            fragment_map,
            selection_sets_out,
            inline_fragments_out,
            variables: variable_definitions
                .iter()
                .map(|var| Variable::from(var, type_index))
                .collect(),
        }
    }

    fn normalise_object_selection_set(
        &mut self,
        selection_set: &parser::SelectionSet<'query>,
        current_path: GraphPath<'query>,
    ) -> Result<Rc<SelectionSet<'query, 'schema>>, Error> {
        let current_type = self.type_index.type_for_path(&current_path)?;

        let mut selections = Vec::new();

        for item in &selection_set.items {
            selections.extend(self.convert_selection(item, &current_path)?);
        }

        let rv = Rc::new(SelectionSet {
            target_type: current_type.try_into()?,
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

                let inner_field = match schema_field.value_type.inner_ref().lookup()? {
                    OutputType::Object(_) if field.selection_set.items.is_empty() => {
                        return Err(Error::NoFieldSelected(schema_field.name.into()));
                    }
                    OutputType::Object(_) => Field::Composite(
                        self.normalise_object_selection_set(&field.selection_set, new_path)?,
                    ),
                    OutputType::Interface(_) | OutputType::Union(_) => {
                        self.normalise_abstract_selection_set(&field.selection_set, new_path)?
                    }
                    OutputType::Enum(_) | OutputType::Scalar(_) => Field::Leaf,
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

                let TypeCondition::On(target_type_name) = fragment.type_condition;
                let current_type = self.type_index.type_for_path(current_path)?;
                let target_type = self.type_index.lookup_type(target_type_name)?;

                current_type.allows_fragment_target_of(&target_type)?;

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
                    let current_type = self.type_index.type_for_path(current_path)?;
                    let target_type = self.type_index.lookup_type(target_type_name)?;

                    current_type.allows_fragment_target_of(&target_type)?;
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
        }
    }

    fn normalise_abstract_selection_set(
        &mut self,
        selection_set: &parser::SelectionSet<'query>,
        current_path: GraphPath<'query>,
    ) -> Result<Field<'query, 'schema>, Error> {
        let schema_field = self.type_index.field_for_path(&current_path)?;

        let spread_selections = selection_set
            .items
            .iter()
            .filter(|s| !matches!(s, parser::Selection::Field(_)))
            .collect::<Vec<_>>();

        if spread_selections.is_empty() {
            // No spreads, so lets just treat this like an object
            return Ok(Field::Composite(
                self.normalise_object_selection_set(selection_set, current_path)?,
            ));
        }

        let non_spread_selections = selection_set
            .items
            .iter()
            .filter(|s| matches!(s, parser::Selection::Field(_)))
            .cloned()
            .collect::<Vec<_>>();

        let mut fragment_selections = vec![];

        let schema_field_type =
            TypeRef::from(schema_field.value_type.inner_ref().to_owned()).lookup()?;

        for selection in spread_selections {
            match selection {
                parser::Selection::FragmentSpread(spread) => {
                    let fragment = self
                        .fragment_map
                        .get(spread.fragment_name)
                        .ok_or_else(|| Error::UnknownFragment(spread.fragment_name.to_string()))?;

                    let parser::TypeCondition::On(target_type) = fragment.type_condition;

                    schema_field_type
                        .allows_fragment_target_of(&self.type_index.lookup_type(target_type)?)?;

                    let mut selection_set = fragment.selection_set.clone();

                    selection_set.items.extend(non_spread_selections.clone());

                    fragment_selections.push(self.normalise_object_selection_set(
                        &selection_set,
                        GraphPath::for_named_type(target_type),
                    )?)
                }
                parser::Selection::InlineFragment(inline_fragment) => {
                    let target_type = match inline_fragment.type_condition {
                        None => return Err(Error::MissingTypeCondition),
                        Some(parser::TypeCondition::On(target_type)) => target_type,
                    };

                    schema_field_type
                        .allows_fragment_target_of(&self.type_index.lookup_type(target_type)?)?;

                    let mut selection_set = inline_fragment.selection_set.clone();

                    selection_set.items.extend(non_spread_selections.clone());

                    fragment_selections.push(self.normalise_object_selection_set(
                        &selection_set,
                        GraphPath::for_named_type(target_type),
                    )?)
                }
                parser::Selection::Field(_) => panic!("This should be unreachable"),
            };
        }

        let inline_fragments = Rc::new(InlineFragments {
            abstract_type: schema_field_type,
            inner_selections: fragment_selections,
        });

        if let Some(existing_value) = self.inline_fragments_out.get(&inline_fragments) {
            return Ok(Field::InlineFragments(Rc::clone(existing_value)));
        }

        self.inline_fragments_out
            .insert(Rc::clone(&inline_fragments));

        return Ok(Field::InlineFragments(inline_fragments));
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
                }) => Some(Rc::clone(selection_set)),
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
                let Selection::Field(field) = selection;

                if let Field::Leaf = &field.field {
                    return Some(field.schema_field.value_type.inner_ref().clone());
                }
                None
            })
            .collect()
    }

    pub fn required_input_types(&self) -> Vec<InputTypeRef<'schema>> {
        self.selections
            .iter()
            .flat_map(|selection| {
                let Selection::Field(field) = selection;

                field
                    .arguments
                    .iter()
                    .map(|(_, arg)| arg.value_type().inner_ref().clone())
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

impl<'query, 'schema> crate::naming::Nameable for Rc<SelectionSet<'query, 'schema>> {
    fn requested_name(&self) -> String {
        self.target_type.name().to_owned()
    }
}

impl<'query, 'schema> crate::naming::Nameable for Rc<InlineFragments<'query, 'schema>> {
    fn requested_name(&self) -> String {
        self.abstract_type.name().to_owned()
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
