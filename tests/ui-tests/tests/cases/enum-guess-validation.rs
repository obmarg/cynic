fn main() {}

mod query_dsl {
    cynic::query_dsl!("../../../cynic/src/bin/simple.graphql");
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(
    schema_path = "../../../cynic/src/bin/simple.graphql",
    query_module = "query_dsl",
    graphql_type = "Desseqt"
)]
enum Dessert {
    CHEESECAKE,
    ICE_CREAM,
}
