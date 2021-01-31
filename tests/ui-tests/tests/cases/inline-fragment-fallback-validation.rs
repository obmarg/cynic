fn main() {}

mod query_dsl {
    type Json = serde_json::Value;

    cynic::query_dsl!("../../../cynic/src/bin/simple.graphql");
}

#[derive(cynic::InlineFragments)]
#[cynic(
    schema_path = "../../../cynic/src/bin/simple.graphql",
    query_module = "query_dsl",
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
    query_module = "query_dsl",
    graphql_type = "Nested"
)]
struct Nested {
    pub a_string: String,
    pub opt_string: Option<String>,
}

#[derive(cynic::InlineFragments)]
#[cynic(
    schema_path = "../../../cynic/src/bin/simple.graphql",
    query_module = "query_dsl",
    graphql_type = "MyUnionType"
)]
enum MyOkUnionTYpe {
    Nested(Nested),

    #[cynic(fallback)]
    Other,
}
