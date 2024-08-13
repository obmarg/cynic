fn main() {}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Root")]
pub struct AllFilms {
    pub __typename: String,
}

#[cynic::schema("other")]
mod schema {}
