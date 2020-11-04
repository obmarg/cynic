use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};

use crate::{
    query::{self, Definition, Document, FragmentDefinition, OperationDefinition},
    Error, GraphPath, TypeIndex,
};

struct NormalisedOperation<'a> {
    root: Rc<SelectionSet<'a>>,
}

#[derive(Hash, PartialEq, Eq, Debug)]
struct SelectionSet<'a> {
    target_type: &'a str,
    selections: Vec<Selection<'a>>,
}

#[derive(Hash, PartialEq, Eq, Debug)]
enum Selection<'a> {
    // For now I just care about fields
    // Will probably need InlineFragments here sometime
    // Figure a normal FragmentSpread can be normalised in place.
    Field(FieldSelection<'a>),
}

#[derive(Debug)]
struct FieldSelection<'a> {
    alias: Option<&'a str>,
    name: &'a str,
    arguments: Vec<(&'a str, HashableValue<'a>)>,
    //  Problem here is we can't just store Value as that isn't hashable...
    //  So either some hashable wrapper type
    //  or a full translation (probably the former)
    selection_set: Weak<SelectionSet<'a>>,

    // Weak is not hashable so we need to take a hash when we create
    // the FieldSelection
    hash: u64,
}

impl<'a> FieldSelection<'a> {
    fn new(
        name: &'a str,
        alias: Option<&'a str>,
        arguments: &'a [(&'a str, query::Value<'a>)],
        selection_set: &Rc<SelectionSet<'a>>,
    ) -> FieldSelection<'a> {
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

impl<'a> Hash for FieldSelection<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl<'a> PartialEq for FieldSelection<'a> {
    fn eq(&self, other: &FieldSelection) -> bool {
        // TODO: Should probably implement an actual equals here...
        self.hash == other.hash
    }
}

impl<'a> Eq for FieldSelection<'a> {}

#[derive(PartialEq, Debug)]
struct HashableValue<'a> {
    inner: &'a query::Value<'a>,
}

impl<'a> HashableValue<'a> {
    fn new(inner: &'a query::Value<'a>) -> Self {
        HashableValue { inner }
    }
}

// Note: Technically this is wrong - a HashableValue _could_
// contain a floating point which is not Eq.
//
// But in practice I hope we'll be OK
impl<'a> Eq for HashableValue<'a> {}

impl<'a> Hash for HashableValue<'a> {
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

type SelectionSetSet<'a> = HashSet<Rc<SelectionSet<'a>>>;

struct NormalisedDocument<'a> {
    selection_sets: SelectionSetSet<'a>,
    operations: Vec<NormalisedOperation<'a>>,
}

// TODO: Make this (and all the types) public
fn normalise<'a>(
    document: &'a Document<'a>,
    type_index: &'a TypeIndex<'a>,
) -> Result<NormalisedDocument<'a>, Error> {
    let fragment_map = extract_fragments(&document);

    let mut selection_sets: SelectionSetSet<'a> = HashSet::new();
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

fn normalise_operation<'a, 'b>(
    operation: &'a OperationDefinition<'a>,
    fragment_map: &'b FragmentMap<'a>,
    type_index: &'a TypeIndex<'a>,
    selection_sets_out: &'b mut SelectionSetSet<'a>,
) -> Result<NormalisedOperation<'a>, Error> {
    // Ok, so real issue at this point is we don't have type information.
    // So can't really figure out what type each field is.
    // So almost need to walk the tree building that _and then_

    match operation {
        OperationDefinition::SelectionSet(selection_set) => {
            normalise_selection_set(
                &selection_set,
                type_index,
                GraphPath::for_query(),
                selection_sets_out,
            )?;
        }
        OperationDefinition::Query(query) => {
            // TODO: This one will be v similar to the selection set above,
            // just with a potential for ArgumentStructs & Variables
            todo!()
        }
        OperationDefinition::Mutation(mutation) => {
            // TODO: Imagine this one will be exactly the same as query.
            todo!()
        }
        OperationDefinition::Subscription(_) => {
            return Err(Error::UnsupportedQueryDocument(
                "Subscriptions are not yet supported".into(),
            ));
        }
    }

    todo!()
}

fn normalise_selection_set<'a>(
    selection_set: &'a query::SelectionSet<'a>,
    type_index: &'a TypeIndex<'a>,
    current_path: GraphPath,
    selection_sets_out: &mut SelectionSetSet<'a>,
) -> Result<Rc<SelectionSet<'a>>, Error> {
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
        target_type: type_index.type_name_for_path(&current_path)?,
        selections,
    });

    if let Some(existing_value) = selection_sets_out.get(&rv) {
        return Ok(Rc::clone(existing_value));
    }

    selection_sets_out.insert(Rc::clone(&rv));

    Ok(rv)
}

type FragmentMap<'a> = HashMap<&'a str, &'a FragmentDefinition<'a>>;

fn extract_fragments<'a>(document: &'a Document<'a>) -> FragmentMap<'a> {
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
    #[test]
    fn write_some_tests() {
        todo!()
    }
}
