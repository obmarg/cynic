mod parser;
mod type_index;

// TOOD: Should just re-export from this?
pub mod names;
pub mod types;

pub use self::parser::{load_schema, Document};

// TODO: Remove these once we've stopped using the parser types directly.
pub use self::{parser::*, type_index::TypeIndex};

// TODO: Uncomment this
// pub use self::{types::*},

pub struct Schema<'a> {
    doc: &'a Document,
    type_index: type_index::TypeIndex<'a>,
}

impl<'a> Schema<'a> {
    pub fn new(document: &'a Document) -> Self {
        let type_index = type_index::TypeIndex::for_schema_2(document);

        Schema {
            doc: document,
            type_index,
        }
    }

    pub fn lookup<Kind>(&self, name: &str) -> Result<Kind, SchemaError>
    where
        Kind: TryFrom<types::Type<'a>, Error = SchemaError> + 'a,
    {
        Kind::try_from(self.type_index.lookup_valid_type(name)?)
    }
}

#[derive(Debug)]
pub enum SchemaError {
    UnexpectedKind {
        name: String,
        expected: types::Kind,
        found: types::Kind,
    },
    InvalidTypeInSchema {
        name: String,
        details: String,
    },
    CouldNotFindType {
        name: String,
    },
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedKind {
                name,
                expected,
                found,
            } => write!(
                f,
                "Expected type `{}` to be a {} but it was a {}",
                name, expected, found
            ),
            Self::InvalidTypeInSchema { name, details } => {
                write!(f, "Invalid schema when validating `{}`: {}", name, details)
            }
            Self::CouldNotFindType { name } => {
                write!(f, "Could not find a type named `{}` in the schema", name)
            }
        }
    }
}

impl From<SchemaError> for crate::Errors {
    fn from(val: SchemaError) -> Self {
        syn::Error::new(proc_macro2::Span::call_site(), val).into()
    }
}

impl SchemaError {
    pub fn unexpected_kind(actual: types::Type<'_>, expected: types::Kind) -> Self {
        SchemaError::UnexpectedKind {
            name: actual.name().to_string(),
            expected,
            found: actual.kind(),
        }
    }
}
