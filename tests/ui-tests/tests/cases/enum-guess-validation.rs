fn main() {}

mod schema {
    cynic::use_schema!("../../../cynic/src/bin/simple.graphql");
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(
    schema_path = "../../../cynic/src/bin/simple.graphql",
    query_module = "schema",
    graphql_type = "Desseqt"
)]
enum Dessert {
    CHEESECAKE,
    ICE_CREAM,
}
