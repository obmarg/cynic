//! TODO: Docstring
use std::collections::HashMap;

use rkyv::Deserialize;

use crate::schema::{
    self,
    types::{SchemaRoots, Type},
    Schema, SchemaError,
};

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, PartialEq, Eq, Debug)]
#[archive(check_bytes)]
/// TODO: Doc string
pub struct OptimisedTypes<'a> {
    types: HashMap<String, Type<'a>>,
    schema_roots: SchemaRoots<'a>,
}

impl Schema<'_, schema::Validated> {
    fn optimise(&self) -> OptimisedTypes<'_> {
        OptimisedTypes {
            types: self
                .type_index
                .unsafe_iter()
                .map(|ty| (ty.name().to_string(), ty))
                .collect(),
            schema_roots: self.type_index.root_types().expect("valid root types"),
        }
    }
}

#[ouroboros::self_referencing]
pub struct ArchiveBacked {
    data: Vec<u8>,
    #[borrows(data)]
    archived: &'this ArchivedOptimisedTypes<'static>,
}

impl ArchiveBacked {
    pub fn from_checked_data(data: Vec<u8>) -> Self {
        ArchiveBacked::new(data, |data| unsafe {
            // This is safe so long as we've already verified data
            rkyv::archived_root::<OptimisedTypes<'_>>(&data[..])
        })
    }
}

impl super::TypeIndex for ArchiveBacked {
    fn validate_all(&self) -> Result<(), SchemaError> {
        // An ArchiveBacked can only be created from already validated types.
        Ok(())
    }

    fn lookup_valid_type<'a>(&'a self, name: &str) -> Result<Type<'a>, SchemaError> {
        Ok(self
            .borrow_archived()
            .types
            .get(name)
            .ok_or_else(|| SchemaError::CouldNotFindType {
                name: name.to_string(),
            })?
            .deserialize(&mut rkyv::Infallible)
            .expect("infalliable"))
    }

    fn root_types(&self) -> Result<schema::types::SchemaRoots<'_>, SchemaError> {
        Ok(self
            .borrow_archived()
            .schema_roots
            .deserialize(&mut rkyv::Infallible)
            .expect("infallible"))
    }

    fn unsafe_lookup<'a>(&'a self, name: &str) -> Option<Type<'a>> {
        Some(
            self.borrow_archived()
                .types
                .get(name)
                .unwrap()
                .deserialize(&mut rkyv::Infallible)
                .expect("infallible"),
        )
    }

    fn unsafe_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Type<'a>> + 'a> {
        Box::new(self.borrow_archived().types.values().map(|archived_type| {
            archived_type
                .deserialize(&mut rkyv::Infallible)
                .expect("infallible")
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rkyv::Deserialize;

    use crate::schema::{type_index::TypeIndex, SchemaInput};

    use super::*;

    #[test]
    fn test_rkyv_roundtrip() {
        let schema_string = std::fs::read_to_string("../schemas/github.graphql").unwrap();
        let schema = Schema::new(SchemaInput::from_sdl(&schema_string).unwrap())
            .validate()
            .unwrap();

        let optimised = schema.optimise();

        // Make sure optimising hasn't broken anything
        optimised
            .types
            .iter()
            .for_each(|(name, ty)| assert_eq!(schema.lookup::<Type<'_>>(name).unwrap(), *ty));

        let bytes = rkyv::to_bytes::<_, 1024>(&optimised).unwrap();
        let archived = rkyv::check_archived_root::<OptimisedTypes<'_>>(&bytes[..]).unwrap();

        let deserialized: OptimisedTypes<'_> = archived.deserialize(&mut rkyv::Infallible).unwrap();

        assert_eq!(deserialized, optimised);
    }

    #[test]
    fn test_type_index_impl() {
        let schema_string = std::fs::read_to_string("../schemas/github.graphql").unwrap();
        let schema = Schema::new(SchemaInput::from_sdl(&schema_string).unwrap())
            .validate()
            .unwrap();

        let optimised = schema.optimise();

        let bytes = rkyv::to_bytes::<_, 1024>(&optimised).unwrap();

        let schema_backed = schema.type_index;
        let archive_backed = ArchiveBacked::from_checked_data(bytes.to_vec());

        assert_eq!(
            archive_backed.root_types().unwrap(),
            schema_backed.root_types().unwrap()
        );

        for ty in schema_backed.unsafe_iter() {
            assert_eq!(archive_backed.lookup_valid_type(ty.name()).unwrap(), ty);
            assert_eq!(archive_backed.unsafe_lookup(ty.name()).unwrap(), ty);
        }

        let all_schema_backed = schema_backed.unsafe_iter().collect::<HashSet<_>>();
        let all_archive_backed = archive_backed.unsafe_iter().collect::<HashSet<_>>();
        assert_eq!(all_archive_backed, all_schema_backed);
    }
}
