fn main() {}

mod schema {
    cynic::use_schema!("../../../../schemas/simple.graphql");
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(
    schema_path = "../../../../schemas/simple.graphql",
    graphql_type = "Desseqt"
)]
enum Dessert {
    CHEESECAKE,
    ICE_CREAM,
}
