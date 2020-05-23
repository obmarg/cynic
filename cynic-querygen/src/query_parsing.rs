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
                    let mut structs = selection_set_to_structs(&query.selection_set, vec![])?;

                    // selection_set_to_structs traverses the tree in post-order
                    // (sort of), so we reverse to get the root node first.
                    structs.reverse();

                    Ok(structs)
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
