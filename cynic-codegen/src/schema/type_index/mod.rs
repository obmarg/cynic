use super::{
    types::{SchemaRoots, Type},
    SchemaError,
};

pub mod optimised;
mod schema_backed;

pub use self::schema_backed::SchemaBackedTypeIndex;

pub trait TypeIndex {
    fn validate_all(&self) -> Result<(), SchemaError>;
    fn lookup_valid_type<'a>(&'a self, name: &str) -> Result<Type<'a>, SchemaError>;
    fn root_types(&self) -> Result<SchemaRoots<'_>, SchemaError>;

    // These are only safe to call if the TypeIndex has been validated.
    // The Schema should make sure that's the case...
    fn unsafe_lookup<'a>(&'a self, name: &str) -> Option<Type<'a>>;
    fn unsafe_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Type<'a>> + 'a>;
}
