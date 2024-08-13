use cynic::QueryFragment;

mod schema {
    cynic::use_schema!("tests/test-schema.graphql");
}

/// Fields with the name `key` were failing to compile in some versions of cynic, so
/// this struct is a test of that.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(schema_path = "tests/test-schema.graphql")]
struct TypeWithKey {
    key: String,
}

// This one is just a test that you can use the GraphQlErrors type without
// _having_ to specify an Extensions type.
#[allow(dead_code)]
type Error = cynic::GraphQlError;
