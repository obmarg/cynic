fn main() {}

mod schema {
    cynic::use_schema!("../../../../schemas/simple.graphql");
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(
    schema_path = "../../../../schemas/simple.graphql",
    graphql_type = "Desseqt"
)]
#[allow(non_camel_case_types)]
enum Dessert {
    CHEESECAKE,
    ICE_CREAM,
}
