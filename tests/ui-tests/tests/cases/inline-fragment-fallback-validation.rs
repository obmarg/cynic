fn main() {}

mod schema {
    cynic::use_schema!("../../../cynic/src/bin/simple.graphql");
}

#[derive(cynic::InlineFragments)]
#[cynic(
    schema_path = "../../../cynic/src/bin/simple.graphql",
    query_module = "schema",
    graphql_type = "MyUnionType"
)]
enum MyFailingUnionType {
    Nested(Nested),

    #[cynic(fallback)]
    Other(Nested),
}

#[derive(cynic::QueryFragment)]
#[cynic(
    schema_path = "../../../cynic/src/bin/simple.graphql",
    query_module = "schema"
)]
struct Nested {
    pub a_string: String,
    pub opt_string: Option<String>,
}

#[derive(cynic::InlineFragments)]
#[cynic(
    schema_path = "../../../cynic/src/bin/simple.graphql",
    query_module = "schema",
    graphql_type = "MyUnionType"
)]
enum MyOkUnionTYpe {
    Nested(Nested),

    #[cynic(fallback)]
    Other,
}
