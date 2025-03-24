use std::{
    collections::{hash_map::DefaultHasher, BTreeSet, HashMap, HashSet},
    hash::{Hash, Hasher},
    rc::Rc,
};

use cynic_parser::{executable::Iter, ExecutableDocument};
use inflector::Inflector;

use super::{
    parser::{self, FragmentDefinition, OperationDefinition, VariableDefinition},
    sorting::Vertex,
    value::TypedValue,
};

use crate::{
    schema::{InputFieldType, InputTypeRef, OutputField, OutputType, OutputTypeRef, Type, TypeRef},
    Error, GraphPath, TypeIndex,
};

#[derive(Debug, PartialEq, Eq)]
pub struct NormalisedOperation<'query, 'schema> {
    pub root: Rc<SelectionSet<'query, 'schema>>,
    pub name: Option<String>,
    pub variables: Vec<Variable<'query, 'schema>>,
    pub kind: OperationKind,
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Hash)]
pub struct Variable<'query, 'schema> {
    pub name: &'query str,
    pub value_type: InputFieldType<'schema>,
}

#[derive(Debug, PartialEq, Eq)]
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

    pub directives: Vec<(&'schema str, TypedValue<'query, 'schema>)>,

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
        directives: Vec<(&'schema str, TypedValue<'query, 'schema>)>,
        schema_field: OutputField<'schema>,
        field: Field<'query, 'schema>,
    ) -> FieldSelection<'query, 'schema> {
        FieldSelection {
            name,
            alias,
            arguments,
            directives,
            schema_field,
            field,
        }
    }
}

// Use a BTreeSet here as we want a deterministic order of output for a
// given document
type SelectionSetSet<'query, 'schema> = BTreeSet<Rc<SelectionSet<'query, 'schema>>>;
pub type InlineFragmentsSet<'query, 'schema> = BTreeSet<Rc<InlineFragments<'query, 'schema>>>;

#[derive(Debug, PartialEq, Eq)]
pub struct NormalisedDocument<'query, 'schema> {
    pub selection_sets: SelectionSetSet<'query, 'schema>,
    pub inline_fragments: InlineFragmentsSet<'query, 'schema>,
    pub operations: Vec<NormalisedOperation<'query, 'schema>>,
}

pub fn normalise<'docs>(
    document: &'docs ExecutableDocument,
    type_index: &Rc<TypeIndex<'docs>>,
) -> Result<NormalisedDocument<'docs, 'docs>, Error> {
    let fragment_map = extract_fragments(document);

    let mut selection_sets: SelectionSetSet<'docs, 'docs> = BTreeSet::new();
    let mut inline_fragments: InlineFragmentsSet<'docs, 'docs> = BTreeSet::new();
    let mut operations = Vec::new();

    for operation in document.operations() {
        operations.push(normalise_operation(
            operation,
            &fragment_map,
            type_index,
            &mut selection_sets,
            &mut inline_fragments,
        )?);
    }

    Ok(NormalisedDocument {
        selection_sets,
        inline_fragments,
        operations,
    })
}

fn normalise_operation<'docs>(
    operation: OperationDefinition<'docs>,
    fragment_map: &FragmentMap<'docs>,
    type_index: &Rc<TypeIndex<'docs>>,
    selection_sets_out: &mut SelectionSetSet<'docs, 'docs>,
    inline_fragments_out: &mut InlineFragmentsSet<'docs, 'docs>,
) -> Result<NormalisedOperation<'docs, 'docs>, Error> {
    let mut normaliser = Normaliser::new(
        type_index,
        fragment_map,
        selection_sets_out,
        inline_fragments_out,
        operation.variable_definitions(),
    );

    let (kind, starting_path) = match operation.operation_type() {
        cynic_parser::common::OperationType::Query => {
            (OperationKind::Query, GraphPath::for_query())
        }
        cynic_parser::common::OperationType::Mutation => {
            (OperationKind::Mutation, GraphPath::for_mutation())
        }
        cynic_parser::common::OperationType::Subscription => {
            (OperationKind::Subscription, GraphPath::for_subscription())
        }
    };

    let root =
        normaliser.normalise_object_selection_set(operation.selection_set(), starting_path)?;

    Ok(NormalisedOperation {
        root,
        name: operation.name().map(Inflector::to_pascal_case),
        kind,
        variables: normaliser.variables,
    })
}

struct Normaliser<'a, 'docs> {
    type_index: &'a Rc<TypeIndex<'docs>>,
    fragment_map: &'a FragmentMap<'docs>,
    selection_sets_out: &'a mut SelectionSetSet<'docs, 'docs>,
    inline_fragments_out: &'a mut InlineFragmentsSet<'docs, 'docs>,
    variables: Vec<Variable<'docs, 'docs>>,
}

impl<'a, 'docs> Normaliser<'a, 'docs> {
    fn new(
        type_index: &'a Rc<TypeIndex<'docs>>,
        fragment_map: &'a FragmentMap<'docs>,
        selection_sets_out: &'a mut SelectionSetSet<'docs, 'docs>,
        inline_fragments_out: &'a mut InlineFragmentsSet<'docs, 'docs>,
        variable_definitions: Iter<'docs, VariableDefinition<'docs>>,
    ) -> Self {
        Normaliser {
            type_index,
            fragment_map,
            selection_sets_out,
            inline_fragments_out,
            variables: variable_definitions
                .map(|var| Variable::from(var, type_index))
                .collect(),
        }
    }

    fn normalise_object_selection_set(
        &mut self,
        selection_sets: impl Iterator<Item = parser::Selection<'docs>>,
        current_path: GraphPath<'docs>,
    ) -> Result<Rc<SelectionSet<'docs, 'docs>>, Error> {
        let current_type = self.type_index.type_for_path(&current_path)?;

        // Awkwardly using a set of hashes & to dedup so we don't fuck
        // up the order of the selections using a BTreeset or Vec.dedup
        let mut seen_selections = HashSet::new();
        let mut selections = Vec::new();

        for item in selection_sets {
            let new_selections = self.convert_selection(item, &current_path)?;
            for selection in new_selections {
                let mut hasher = DefaultHasher::new();
                selection.hash(&mut hasher);
                let hash = hasher.finish();
                if seen_selections.contains(&hash) {
                    continue;
                }
                seen_selections.insert(hash);
                selections.push(selection);
            }
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
        selection: parser::Selection<'docs>,
        current_path: &GraphPath<'docs>,
    ) -> Result<Vec<Selection<'docs, 'docs>>, Error> {
        match selection {
            parser::Selection::Field(field) => {
                let new_path = current_path.push(field.name());

                let schema_field = self.type_index.field_for_path(&new_path)?;

                let inner_field = match schema_field.value_type.inner_ref().lookup()? {
                    OutputType::Object(_) if field.selection_set().len() == 0 => {
                        return Err(Error::NoFieldSelected(schema_field.name.into()));
                    }
                    OutputType::Object(_) => Field::Composite(
                        self.normalise_object_selection_set(field.selection_set(), new_path)?,
                    ),
                    OutputType::Interface(_) | OutputType::Union(_) => {
                        self.normalise_abstract_selection_set(field.selection_set(), new_path)?
                    }
                    OutputType::Enum(_) | OutputType::Scalar(_) => Field::Leaf,
                };

                let mut arguments = Vec::new();
                for argument in field.arguments() {
                    let name = argument.name();
                    let value = argument.value();

                    let schema_arg = schema_field
                        .arguments
                        .iter()
                        .find(|arg| arg.name == name)
                        .ok_or_else(|| dbg!(Error::UnknownArgument(name.to_string())))?;

                    arguments.push((
                        schema_arg.name,
                        TypedValue::from_query_value(
                            value,
                            schema_arg.value_type.clone(),
                            &self.variables,
                        )?,
                    ));
                }

                let directives = field
                    .directives()
                    .map(|directive| {
                        let name = directive.name();
                        let schema_directive = self.type_index.directive(name)?;

                        (schema_directive.name(), todo!())
                    })
                    .collect::<Result<_, _>>()?;

                Ok(vec![Selection::Field(FieldSelection::new(
                    field.name(),
                    field.alias(),
                    arguments,
                    directives,
                    schema_field,
                    inner_field,
                ))])
            }
            parser::Selection::FragmentSpread(spread) => {
                let fragment = self
                    .fragment_map
                    .get(spread.fragment_name())
                    .ok_or_else(|| Error::UnknownFragment(spread.fragment_name().to_string()))?;

                let current_type = self.type_index.type_for_path(current_path)?;
                let target_type = self.type_index.lookup_type(fragment.type_condition())?;

                current_type.allows_fragment_target_of(&target_type)?;

                Ok(fragment
                    .selection_set()
                    .map(|item| self.convert_selection(item, current_path))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .flatten()
                    .collect())
            }
            parser::Selection::InlineFragment(fragment) => {
                if let Some(target_type_name) = fragment.type_condition() {
                    let current_type = self.type_index.type_for_path(current_path)?;
                    let target_type = self.type_index.lookup_type(target_type_name)?;

                    current_type.allows_fragment_target_of(&target_type)?;
                }

                Ok(fragment
                    .selection_set()
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
        selection_set: Iter<'docs, parser::Selection<'docs>>,
        current_path: GraphPath<'docs>,
    ) -> Result<Field<'docs, 'docs>, Error> {
        let schema_field = self.type_index.field_for_path(&current_path)?;

        let spread_selections = selection_set
            .clone()
            .filter(|s| !matches!(s, parser::Selection::Field(_)))
            .collect::<Vec<_>>();

        if spread_selections.is_empty() {
            // No spreads, so lets just treat this like an object
            return Ok(Field::Composite(
                self.normalise_object_selection_set(selection_set, current_path)?,
            ));
        }

        let non_spread_selections = selection_set
            .filter(|s| matches!(s, parser::Selection::Field(_)))
            .collect::<Vec<_>>();

        let mut fragment_selections = vec![];

        let schema_field_type =
            TypeRef::from(schema_field.value_type.inner_ref().to_owned()).lookup()?;

        for selection in spread_selections {
            match selection {
                parser::Selection::FragmentSpread(spread) => {
                    let fragment =
                        self.fragment_map
                            .get(spread.fragment_name())
                            .ok_or_else(|| {
                                Error::UnknownFragment(spread.fragment_name().to_string())
                            })?;

                    schema_field_type.allows_fragment_target_of(
                        &self.type_index.lookup_type(fragment.type_condition())?,
                    )?;

                    let selections = fragment
                        .selection_set()
                        .chain(non_spread_selections.iter().copied());

                    fragment_selections.push(self.normalise_object_selection_set(
                        selections,
                        GraphPath::for_named_type(fragment.type_condition()),
                    )?)
                }
                parser::Selection::InlineFragment(inline_fragment) => {
                    let target_type = match inline_fragment.type_condition() {
                        None => return Err(Error::MissingTypeCondition),
                        Some(target_type) => target_type,
                    };

                    schema_field_type
                        .allows_fragment_target_of(&self.type_index.lookup_type(target_type)?)?;

                    let selections = inline_fragment
                        .selection_set()
                        .chain(non_spread_selections.iter().copied());

                    fragment_selections.push(self.normalise_object_selection_set(
                        selections,
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

        Ok(Field::InlineFragments(inline_fragments))
    }
}

impl<'a> Variable<'a, 'a> {
    fn from(def: VariableDefinition<'a>, type_index: &Rc<TypeIndex<'a>>) -> Self {
        Variable {
            name: def.name(),
            value_type: InputFieldType::from_variable_definition(def, type_index),
        }
    }
}

type FragmentMap<'query> = HashMap<&'query str, FragmentDefinition<'query>>;

fn extract_fragments(document: &ExecutableDocument) -> FragmentMap<'_> {
    document
        .fragments()
        .flat_map(|fragment| Some((fragment.name(), fragment)))
        .collect()
}

impl Vertex for SelectionSet<'_, '_> {
    fn children(self: &Rc<Self>) -> Vec<Rc<Self>> {
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

impl<'schema> SelectionSet<'_, 'schema> {
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

impl crate::naming::Nameable for Rc<SelectionSet<'_, '_>> {
    fn requested_name(&self) -> String {
        self.target_type.name().to_pascal_case()
    }
}

impl crate::naming::Nameable for Rc<InlineFragments<'_, '_>> {
    fn requested_name(&self) -> String {
        self.abstract_type.name().to_pascal_case()
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::schema::add_builtins,
        cynic_parser::{type_system::ids::FieldDefinitionId, TypeSystemDocument},
        std::sync::LazyLock,
    };

    use assert_matches::assert_matches;

    #[test]
    fn normalise_deduplicates_identical_selections() {
        let (schema, typename_id) = &*SCHEMA;
        let type_index = Rc::new(TypeIndex::from_schema(schema, *typename_id));
        let query = cynic_parser::parse_executable_document(
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
        let (schema, typename_id) = &*SCHEMA;
        let type_index = Rc::new(TypeIndex::from_schema(schema, *typename_id));
        let query = cynic_parser::parse_executable_document(
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
        let (schema, typename_id) = &*SCHEMA;
        let type_index = Rc::new(TypeIndex::from_schema(schema, *typename_id));
        let query = cynic_parser::parse_executable_document(
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
        let (schema, typename_id) = &*SCHEMA;
        let type_index = Rc::new(TypeIndex::from_schema(schema, *typename_id));
        let query = cynic_parser::parse_executable_document(
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

        insta::assert_debug_snapshot!(film_selections.first().unwrap().selections);
    }

    #[test]
    fn check_fragment_type_mismatches() {
        let (schema, typename_id) = &*SCHEMA;
        let type_index = Rc::new(TypeIndex::from_schema(schema, *typename_id));
        let query = cynic_parser::parse_executable_document(
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
        let (schema, typename_id) = &*SCHEMA;
        let type_index = Rc::new(TypeIndex::from_schema(schema, *typename_id));
        let query = cynic_parser::parse_executable_document(
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
        let (schema, typename_id) = &*SCHEMA;
        let type_index = Rc::new(TypeIndex::from_schema(schema, *typename_id));
        let query = cynic_parser::parse_executable_document(
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
        let (schema, typename_id) = &*SCHEMA;
        let type_index = Rc::new(TypeIndex::from_schema(schema, *typename_id));
        let query = cynic_parser::parse_executable_document(
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

        insta::assert_debug_snapshot!(film_selections.first().unwrap().selections);
    }

    #[test]
    fn check_inline_fragment_type_mismatches() {
        let (schema, typename_id) = &*SCHEMA;
        let type_index = Rc::new(TypeIndex::from_schema(schema, *typename_id));
        let query = cynic_parser::parse_executable_document(
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

    static SCHEMA: LazyLock<(TypeSystemDocument, FieldDefinitionId)> = LazyLock::new(|| {
        let schema = cynic_parser::parse_type_system_document(include_str!(
            "../../../schemas/starwars.schema.graphql"
        ))
        .unwrap();
        add_builtins(schema)
    });
}
