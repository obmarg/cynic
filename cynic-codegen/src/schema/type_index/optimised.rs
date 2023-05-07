//! TODO: Docstring
use std::collections::HashMap;

use crate::schema::{self, types::Type, Schema};

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, PartialEq, Eq, Debug)]
#[archive(check_bytes)]
/// TODO: Doc string
pub struct OptimisedTypes<'a>(HashMap<String, Type<'a>>);

impl Schema<'_, schema::Validated> {
    fn optimise<'a>(&'a self) -> OptimisedTypes<'a> {
        OptimisedTypes(
            self.type_index
                .unsafe_iter()
                .map(|ty| (ty.name().to_string(), ty))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use rkyv::Deserialize;

    use crate::schema::SchemaInput;

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
            .0
            .iter()
            .for_each(|(name, ty)| assert_eq!(schema.lookup::<Type<'_>>(name).unwrap(), *ty));

        let bytes = rkyv::to_bytes::<_, 1024>(&optimised).unwrap();
        let archived = rkyv::check_archived_root::<OptimisedTypes<'_>>(&bytes[..]).unwrap();

        let deserialized: OptimisedTypes<'_> = archived.deserialize(&mut rkyv::Infallible).unwrap();

        assert_eq!(deserialized, optimised);
    }
}
