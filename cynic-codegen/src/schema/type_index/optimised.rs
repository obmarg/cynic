use std::collections::HashMap;

use crate::schema::{self, types::Type, Schema};

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, PartialEq, Eq, Debug)]
#[archive(check_bytes)]
// TODO: Convert into this somehow,
// then serialize
// then write code to load an ArchivedTypes and deserialize
// on appropriate calls...
struct Types<'a>(HashMap<String, Type<'a>>);

impl Schema<'_, schema::Validated> {
    fn optimise<'a>(&'a self) -> Types<'a> {
        Types(
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

    use super::*;

    #[test]
    fn test_rkyv_roundtrip() {
        let schema_string = std::fs::read_to_string("../schemas/github.graphql").unwrap();
        let document = graphql_parser::parse_schema(&schema_string)
            .unwrap()
            .into_static();
        let schema = Schema::new(&document).validate().unwrap();

        let optimised = schema.optimise();

        // Make sure optimising hasn't broken anything
        optimised
            .0
            .iter()
            .for_each(|(name, ty)| assert_eq!(schema.lookup::<Type<'_>>(&name).unwrap(), *ty));

        let bytes = rkyv::to_bytes::<_, 1024>(&optimised).unwrap();
        let archived = rkyv::check_archived_root::<Types<'_>>(&bytes[..]).unwrap();

        let deserialized: Types<'_> = archived.deserialize(&mut rkyv::Infallible).unwrap();

        assert_eq!(deserialized, optimised);
    }
}
