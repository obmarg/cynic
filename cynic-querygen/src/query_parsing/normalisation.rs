use std::{
    collections::{BTreeSet, HashMap, HashSet},
    convert::TryInto,
    hash::Hash,
    rc::Rc,
};

use cynic_parser::{
    ast::{
        self, AstNode, Document, ExecutableDef, FragmentDef, FragmentSpread, Name, NameOwner,
        OperationDef, VariableDef,
    },
    SyntaxNode,
};

use super::{sorting::Vertex, value::TypedValue};

use crate::{
    schema::{InputFieldType, InputTypeRef, OutputField, OutputType, OutputTypeRef},
    Error, GraphPath, TypeIndex,
};

#[derive(Debug, PartialEq)]
pub struct NormalisedOperation<'schema> {
    pub root: Rc<SelectionSet<'schema>>,
    pub name: Option<Name>,
    pub variables: Vec<Variable<'schema>>,
    pub kind: OperationKind,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct Variable<'schema> {
    pub name: String,
    pub value_type: InputFieldType<'schema>,
}

#[derive(Debug, PartialEq)]
pub enum OperationKind {
    Query,
    Mutation,
}

// TODO: under the new scheme maybe this should be QueryFragment...
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SelectionSet<'schema> {
    pub target_type: OutputType<'schema>,
    pub selections: Vec<Selection<'schema>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Selection<'schema> {
    // For now I just care about fields
    // Will probably need InlineFragments here sometime
    // Figure a normal FragmentSpread can be normalised in place.
    Field(FieldSelection<'schema>),
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FieldSelection<'schema> {
    pub alias: Option<Name>,
    pub name: String,

    pub schema_field: OutputField<'schema>,

    pub arguments: Vec<(String, TypedValue<'schema>)>,

    pub field: Field<'schema>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Field<'schema> {
    /// A composite field contains another selection set.
    Composite(Rc<SelectionSet<'schema>>),

    /// A leaf field just contains it's type as a string
    Leaf,
}

impl<'doc, 'schema> FieldSelection<'schema> {
    fn new(
        name: String,
        alias: Option<Name>,
        arguments: Vec<(String, TypedValue<'schema>)>,
        schema_field: OutputField<'schema>,
        field: Field<'schema>,
    ) -> FieldSelection<'schema> {
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
type SelectionSetSet<'schema> = BTreeSet<Rc<SelectionSet<'schema>>>;

#[derive(Debug, PartialEq)]
pub struct NormalisedDocument<'schema> {
    pub selection_sets: SelectionSetSet<'schema>,
    pub operations: Vec<NormalisedOperation<'schema>>,
}

pub fn normalise<'doc, 'schema>(
    document: Document,
    type_index: &'doc Rc<TypeIndex<'schema>>,
) -> Result<NormalisedDocument<'schema>, Error> {
    let fragment_map = extract_fragments(&document);

    let mut selection_sets: SelectionSetSet<'schema> = BTreeSet::new();
    let mut operations = Vec::new();

    for definition in document.definitions() {
        if let ExecutableDef::OperationDef(operation) = definition {
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

fn normalise_operation<'doc, 'schema>(
    operation: OperationDef,
    fragment_map: &FragmentMap,
    type_index: &'doc Rc<TypeIndex<'schema>>,
    selection_sets_out: &mut SelectionSetSet<'schema>,
) -> Result<NormalisedOperation<'schema>, Error> {
    let mut normaliser = Normaliser::new(
        type_index,
        fragment_map,
        selection_sets_out,
        operation
            .variable_defs()
            .map(|v| v.variable_def().collect())
            .unwrap_or_default(),
    );

    if let Some(selection_set) = operation.selection_set() {
        let root = normaliser.normalise_selection_set(selection_set)?;

        return Ok(NormalisedOperation {
            root,
            name: operation.name(),
            kind: operation.operation_type().into(),
            variables: normaliser.variables,
        });
    }

    todo!()
    // Error on operation without a selection set, also on subscriptions:
    /*
    TODO:
        OperationDefinition::Subscription(_) => Err(Error::UnsupportedQueryDocument(
            "Subscriptions are not yet supported".into(),
        )),
        */
}

struct Normaliser<'a, 'schema> {
    type_index: &'a Rc<TypeIndex<'schema>>,
    fragment_map: &'a FragmentMap,
    selection_sets_out: &'a mut SelectionSetSet<'schema>,
    variables: Vec<Variable<'schema>>,

    selection_set_index: HashMap<SyntaxNode, Rc<SelectionSet<'schema>>>,

    // TODO: Associate errors with syntax nodes.
    errors: Vec<Error>,
}

impl<'a, 'schema> Normaliser<'a, 'schema> {
    fn new(
        type_index: &'a Rc<TypeIndex<'schema>>,
        fragment_map: &'a FragmentMap,
        selection_sets_out: &'a mut SelectionSetSet<'schema>,
        variable_definitions: Vec<VariableDef>,
    ) -> Self {
        Normaliser {
            type_index,
            fragment_map,
            selection_sets_out,
            variables: variable_definitions
                .iter()
                .map(|var| Variable::from(var, type_index))
                .collect(),
            selection_set_index: HashMap::new(),
            errors: vec![],
        }
    }

    fn add_selection_set(
        &mut self,
        set: SelectionSet<'schema>,
        ast: ast::SelectionSet,
    ) -> Rc<SelectionSet<'schema>> {
        let set = Rc::new(set);

        self.selection_sets_out.insert(Rc::clone(&set));
        let set = self.selection_sets_out.get(&set).unwrap();

        self.selection_set_index
            .insert(ast.syntax().clone(), Rc::clone(set));

        Rc::clone(set)
    }

    fn normalise_selection_set(
        &mut self,
        selection_set: ast::SelectionSet,
    ) -> Result<Rc<SelectionSet<'schema>>, Error> {
        let mut selections = Vec::new();

        // TODO: Figure out what kind of selection set this is.
        if selection_set.selections().any(|s| s.is_inline_fragment()) {
            panic!("Inline fragments not selected yet.  TODO: return an error")
        }

        // TODO: handle empty selection
        for item in selection_set.selections() {
            selections.extend(self.convert_selection(item)?);
        }

        let rv = SelectionSet {
            target_type: self
                .type_index
                .type_for_path(GraphPath::from_query_node(&selection_set))?
                .try_into()?,
            selections,
        };

        Ok(self.add_selection_set(rv, selection_set))
    }

    fn convert_selection(
        &mut self,
        selection: ast::Selection,
    ) -> Result<Vec<Selection<'schema>>, Error> {
        // TODO: Consider just returning option?
        match selection {
            ast::Selection::FieldSelection(field) => {
                let schema_field = self
                    .type_index
                    .field_for_path(GraphPath::from_query_node(&field))?;

                let inner_field =
                    if let OutputType::Object(_) = schema_field.value_type.inner_ref().lookup()? {
                        if let Some(selection_set) = field.selection_set() {
                            // TODO: just look up the result in selection_set_index here...
                            Field::Composite(self.normalise_selection_set(selection_set)?)
                        } else {
                            // TODO: just append error
                            return Err(Error::NoFieldSelected(schema_field.name.into()));
                        }
                    } else {
                        Field::Leaf
                    };

                let mut arguments = Vec::new();
                for argument in field.arguments() {
                    let argument_name = argument.name().expect("TODO").to_string();

                    let schema_arg = schema_field
                        .arguments
                        .iter()
                        .find(|arg| arg.name == argument_name)
                        .ok_or_else(|| Error::UnknownArgument(argument_name.to_string()))?;

                    arguments.push((
                        argument_name,
                        TypedValue::from_query_value(
                            argument.value().expect("TODO: missing val"),
                            schema_arg.value_type.clone(),
                            &self.variables,
                        )?,
                    ));
                }

                Ok(vec![Selection::Field(FieldSelection::new(
                    field
                        .name()
                        .expect("TODO: Make this not expect")
                        .to_string(),
                    field.alias().and_then(|a| a.name()),
                    arguments,
                    schema_field,
                    inner_field,
                ))])
            }
            ast::Selection::FragmentSpread(spread) => {
                let fragment = spread
                    .name()
                    .and_then(|n| self.fragment_map.get(n.text()))
                    .ok_or_else(|| {
                        Error::UnknownFragment(spread.name().expect("TODO no expect").to_string())
                    })?;

                let current_type = self
                    .type_index
                    .type_name_for_path(GraphPath::from_query_node(&spread))?;

                if !fragment.applies_to_type(&current_type) {
                    return Err(Error::TypeConditionFailed(
                        fragment
                            .type_condition()
                            .expect("TODO no expect")
                            .named_type()
                            .expect("TODO: no expect")
                            .name()
                            .expect("TODO: no expect")
                            .to_string(),
                        current_type.to_string(),
                    ));
                }

                let selections = fragment
                    .selection_set()
                    .map(|s| s.selections().collect::<Vec<_>>())
                    .unwrap_or_default();

                Ok(selections
                    .into_iter()
                    .map(|item| self.convert_selection(item))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .flatten()
                    .collect())
            }
            ast::Selection::InlineFragment(fragment) => {
                // TODO: Need a way to attach errors to a particular SyntaxNode,
                // and then collect errors rather than just erroring out.
                // Some way to integrate this with `Options` would be ideal...
                if let Some(type_condition) = fragment.type_condition() {
                    let current_type = self
                        .type_index
                        .type_name_for_path(GraphPath::from_query_node(&fragment))?;

                    let condition_type = type_condition
                        .named_type()
                        .expect("TODO")
                        .name()
                        .expect("TODO");

                    if current_type != condition_type.text() {
                        return Err(Error::TypeConditionFailed(
                            condition_type.text().to_string(),
                            current_type.to_string(),
                        ));
                    }
                }

                let selections = fragment
                    .selection_set()
                    .map(|s| s.selections().collect::<Vec<_>>())
                    .unwrap_or_default();

                Ok(selections
                    .into_iter()
                    .map(|item| self.convert_selection(item))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .flatten()
                    .collect())
            }
        }
    }
}

impl<'schema> Variable<'schema> {
    fn from(def: &VariableDef, type_index: &Rc<TypeIndex<'schema>>) -> Self {
        Variable {
            name: def
                .variable()
                .expect("TODO")
                .name()
                .expect("TODO: No expect")
                .to_string(),
            value_type: InputFieldType::from_variable_definition(def, type_index),
        }
    }
}

type FragmentMap = HashMap<String, FragmentDef>;

fn extract_fragments(document: &Document) -> FragmentMap {
    document
        .definitions()
        .flat_map(|definition| {
            let fragment = definition.fragment_def()?;

            Some((fragment.name()?.to_string(), fragment))
        })
        .collect()
}

impl<'schema> Vertex for Rc<SelectionSet<'schema>> {
    fn adjacents(&self) -> Vec<Self> {
        self.selections
            .iter()
            .flat_map(|selection| match selection {
                Selection::Field(FieldSelection {
                    field: Field::Composite(selection_set),
                    ..
                }) => Some(selection_set),
                _ => None,
            })
            .cloned()
            .collect()
    }
}

impl<'schema> SelectionSet<'schema> {
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

impl<'schema> crate::naming::Nameable for Rc<SelectionSet<'schema>> {
    fn requested_name(&self) -> String {
        self.target_type.name().to_owned()
    }
}

impl From<Option<ast::OperationType>> for OperationKind {
    fn from(val: Option<ast::OperationType>) -> Self {
        if val.is_none() {
            return OperationKind::Query;
        }
        let op = val.unwrap();
        if op.query_keyword_token().is_some() {
            return OperationKind::Query;
        }
        if op.mutation_keyword_token().is_some() {
            return OperationKind::Mutation;
        }
        if op.subscription_keyword_token().is_some() {
            todo!()
        }
        OperationKind::Query
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
        let query = cynic_parser::parse_query_document(
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

        let normalised = normalise(query, &type_index).unwrap();

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
        let query = cynic_parser::parse_query_document(
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

        let normalised = normalise(query, &type_index).unwrap();

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
        let query = cynic_parser::parse_query_document(
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

        let normalised = normalise(query, &type_index).unwrap();

        insta::assert_debug_snapshot!(normalised);
    }

    #[test]
    fn check_fragment_spread_output() {
        let schema = load_schema();
        let type_index = Rc::new(TypeIndex::from_schema(&schema));
        let query = cynic_parser::parse_query_document(
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

        let normalised = normalise(query, &type_index).unwrap();

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
        let query = cynic_parser::parse_query_document(
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
            normalise(query, &type_index),
            Err(Error::TypeConditionFailed(_, _))
        )
    }

    #[test]
    fn check_field_selected() {
        let schema = load_schema();
        let type_index = Rc::new(TypeIndex::from_schema(&schema));
        let query = cynic_parser::parse_query_document(
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
        let normalised = normalise(query, &type_index).unwrap();

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
        let query = cynic_parser::parse_query_document(
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
            normalise(query, &type_index),
            Err(Error::NoFieldSelected(_))
        )
    }

    #[test]
    fn check_inline_fragment_output() {
        let schema = load_schema();
        let type_index = Rc::new(TypeIndex::from_schema(&schema));
        let query = cynic_parser::parse_query_document(
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

        let normalised = normalise(query, &type_index).unwrap();

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
        let query = cynic_parser::parse_query_document(
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
            normalise(query, &type_index),
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
