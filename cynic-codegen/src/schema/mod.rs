mod keywords;
mod names;
mod type_index;

pub mod parser;
pub mod types;

mod input;
pub mod markers;

use std::convert::Infallible;
use std::marker::PhantomData;

pub use self::{input::SchemaInput, names::FieldName, parser::load_schema};
use self::{
    type_index::TypeIndex,
    types::{Directive, SchemaRoots},
};

// TODO: Uncomment this
// pub use self::{types::*},

pub struct Unvalidated;
pub struct Validated;

pub struct Schema<'a, State> {
    type_index: Box<dyn type_index::TypeIndex + 'a>,
    phantom: PhantomData<State>,
}

impl<'a> Schema<'a, Unvalidated> {
    pub fn new(input: SchemaInput) -> Self {
        let type_index: Box<dyn TypeIndex> = match input {
            SchemaInput::Document(ast) => {
                Box::new(type_index::SchemaBackedTypeIndex::for_schema(ast))
            }
            #[cfg(feature = "rkyv")]
            SchemaInput::Archive(data) => Box::new(
                type_index::optimised::ArchiveBacked::from_checked_data(data),
            ),
        };

        Schema {
            phantom: PhantomData,
            type_index,
        }
    }
}

impl<'a> Schema<'a, Unvalidated> {
    pub fn validate(self) -> Result<Schema<'a, Validated>, SchemaError> {
        self.type_index.validate_all()?;

        Ok(Schema {
            // doc: self.doc,
            type_index: self.type_index,
            phantom: PhantomData,
        })
    }

    pub fn lookup<'b, Kind>(&'b self, name: &str) -> Result<Kind, SchemaError>
    where
        Kind: TryFrom<types::Type<'b>> + 'b,
        Kind::Error: Into<SchemaError>,
    {
        Kind::try_from(self.type_index.lookup_valid_type(name)?).map_err(Into::into)
        // TODO: Suggestion logic should probably be implemented here (or in type_index)
    }
}

impl<'a> Schema<'a, Validated> {
    pub fn root_types(&self) -> Result<SchemaRoots<'_>, SchemaError> {
        self.type_index.root_types()
    }

    pub fn iter(&self) -> impl Iterator<Item = types::Type<'_>> {
        // unsafe_iter is safe because we're in a validated schema
        self.type_index.unsafe_iter()
    }

    // Looks up a kind that we're not certain is in the validated schema.
    pub fn try_lookup<'b, Kind>(&'b self, name: &str) -> Result<Kind, SchemaError>
    where
        Kind: TryFrom<types::Type<'b>, Error = SchemaError>,
    {
        Kind::try_from(self.type_index.lookup_valid_type(name)?)
        // TODO: Suggestion logic should probably be implemented here (or in type_index)
    }

    pub fn lookup<'b, Kind>(&'b self, name: &str) -> Result<Kind, SchemaError>
    where
        Kind: TryFrom<types::Type<'b>>,
        Kind::Error: Into<SchemaError>,
    {
        // unsafe_lookup is safe because we're validated and this function
        // should only be called if we're sure the type is in the schema
        Kind::try_from(self.type_index.unsafe_lookup(name)).map_err(Into::into)
    }

    pub fn lookup_directive<'b>(&'b self, name: &str) -> Option<Directive<'b>> {
        // unsafe_directive_lookup is safe because we're validated and this function
        // should only be called if we're sure the type is in the schema
        self.type_index.unsafe_directive_lookup(name)
    }

    pub fn directives(&self) -> impl Iterator<Item = Directive<'_>> {
        self.type_index.unsafe_directive_iter()
    }
}

#[derive(Debug)]
pub enum SchemaError {
    UnexpectedKind {
        name: String,
        expected: types::Kind,
        found: types::Kind,
    },
    InvalidDirectiveArgument {
        directive_name: String,
        argument_name: String,
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
                "The type `{}` is a {} in the GraphQL schema but is being used where we expect a {}",
                name, found, expected,
            ),
            Self::InvalidTypeInSchema { name, details } => {
                write!(f, "Invalid schema when validating `{}`: {}", name, details)
            }
            Self::CouldNotFindType { name } => {
                write!(f, "Could not find a type named `{}` in the schema", name)
            }
            Self::InvalidDirectiveArgument { directive_name, argument_name, expected, found } => {
                write!(f, "argument {argument_name} on @{directive_name} is a {found} but needs to be a {expected}")
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

impl From<Infallible> for SchemaError {
    fn from(_: Infallible) -> Self {
        panic!("This can't happen")
    }
}
