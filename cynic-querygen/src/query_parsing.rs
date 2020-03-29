use graphql_parser::query::{Definition, Document, OperationDefinition, Selection, SelectionSet};

use crate::Error;

#[derive(Debug, PartialEq)]
pub struct Field<'a> {
    pub name: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct QueryFragment<'a> {
    pub fields: Vec<Field<'a>>,
    pub path: Vec<&'a str>,
}

#[derive(Debug, PartialEq)]
pub struct InlineFragment {}

#[derive(Debug, PartialEq)]
pub enum PotentialStruct<'a> {
    QueryFragment(QueryFragment<'a>),
    InlineFragment(InlineFragment),
}

pub fn parse_query_document<'a>(
    doc: &'a Document<'a, &'a str>,
) -> Result<Vec<PotentialStruct<'a>>, Error> {
    doc.definitions
        .iter()
        .map(|definition| {
            match definition {
                Definition::Operation(OperationDefinition::Query(query)) => {
                    Ok(selection_set_to_structs(&query.selection_set, vec![])?)
                    // TODO: Some stuff
                    // OK, so the best idea is probably to traverse the query.
                    // By traversing the query we can output a list of the structs we encounter:
                    // their type names, fields selected, positions in query and respective arguments.
                    //
                    // Once we have that list we can group them by type names and _hopefully_ consolidate,
                    // warning if we have too much differentiating.
                }
                Definition::Operation(OperationDefinition::Mutation(_)) => {
                    return Err(Error::UnsupportedQueryDocument(format!(
                        "mutations are not yet supported"
                    )));
                }
                Definition::Operation(OperationDefinition::Subscription(_)) => {
                    return Err(Error::UnsupportedQueryDocument(format!(
                        "subscriptions are not supported"
                    )));
                }
                Definition::Operation(OperationDefinition::SelectionSet(_)) => {
                    return Err(Error::UnsupportedQueryDocument(format!(
                        "top-level selection sets are not yet supported"
                    )));
                }
                Definition::Fragment(_) => {
                    return Err(Error::UnsupportedQueryDocument(format!(
                        "fragments are not yet supported"
                    )));
                }
            }
        })
        .collect::<Result<Vec<Vec<_>>, Error>>()
        .map(|vec| vec.into_iter().flatten().collect())
}

fn selection_set_to_structs<'a>(
    selection_set: &'a SelectionSet<'a, &'a str>,
    path: Vec<&'a str>,
) -> Result<Vec<PotentialStruct<'a>>, Error> {
    // TODO: Ok, so for this to work I basically need a single QueryFragment for all the fields,
    // (and any of their nested QueryFragments).
    //
    // an InlineFragmentDerive for any InlineFragments and errors for any FragmentSpreads...
    // So not sure what I've written here makes any sense...
    let mut rv = Vec::new();

    let path = &path;

    let mut this_fragment = QueryFragment {
        path: path.clone(),
        fields: Vec::new(),
    };

    for item in &selection_set.items {
        match item {
            Selection::Field(field) => {
                this_fragment.fields.push(Field { name: field.name });

                let mut new_path = path.clone();
                new_path.push(field.name);

                rv.extend(selection_set_to_structs(&field.selection_set, new_path)?);
            }
            Selection::FragmentSpread(_) => {
                return Err(Error::UnsupportedQueryDocument(
                    "Fragment spreads are not yet supported".into(),
                ));
            }
            Selection::InlineFragment(_) => {
                return Err(Error::UnsupportedQueryDocument(
                    "Inline fragments are not yet supported".into(),
                ));
            }
        }
    }

    if !this_fragment.fields.is_empty() {
        rv.push(PotentialStruct::QueryFragment(this_fragment));
    }

    Ok(rv)
}
