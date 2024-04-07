use super::{
    types::{Directive, SchemaRoots, Type},
    SchemaError,
};

#[cfg(feature = "rkyv")]
pub mod optimised;

mod schema_backed;

pub use self::schema_backed::SchemaBackedTypeIndex;

pub trait TypeIndex {
    fn validate_all(&self) -> Result<(), SchemaError>;
    fn lookup_valid_type<'a>(&'a self, name: &str) -> Result<Type<'a>, SchemaError>;
    fn root_types(&self) -> Result<SchemaRoots<'_>, SchemaError>;

    // These are only safe to call if the TypeIndex has been validated.
    // The Schema should make sure that's the case...
    fn unsafe_lookup<'a>(&'a self, name: &str) -> Type<'a>;
    fn unsafe_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Type<'a>> + 'a>;
    fn unsafe_directive_lookup<'b>(&'b self, name: &str) -> Option<Directive<'b>>;
    fn unsafe_directive_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Directive<'a>> + 'a>;
}
