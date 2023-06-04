fn main() {}

mod schema {
    cynic::use_schema!("../../../../cynic/src/bin/simple.graphql");
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(
    schema_path = "../../../../cynic/src/bin/simple.graphql",
    graphql_type = "Desseqt"
)]
enum Dessert {
    CHEESECAKE,
    ICE_CREAM,
}
