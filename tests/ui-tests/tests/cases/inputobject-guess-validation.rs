#![allow(unused_imports)]

fn main() {}

#[cynic::query_module(
    schema_path = r#"./../../../schemas/github.graphql"#,
    query_module = "query_dsl"
)]
mod queries {
    use super::{query_dsl, types::*};

    #[derive(cynic::Enum, Clone, Copy, Debug)]
    #[cynic(graphql_type = "IssueOrderField")]
    pub enum IssueOrderField {
        Comments,
        CreatedAt,
        UpdatedAt,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug)]
    #[cynic(graphql_type = "OrderDirection")]
    pub enum OrderDirection {
        Asc,
        Desc,
    }

    #[derive(cynic::InputObject, Debug)]
    #[cynic(graphql_type = "IssueOrder")]
    pub struct IssueOrder {
        pub direction: OrderDirection,
        pub fieid: IssueOrderField,
    }
}

#[cynic::query_module(
    schema_path = r#"./../../../schemas/github.graphql"#,
    query_module = "query_dsl"
)]
mod types {
    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct Date(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct DateTime(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct GitObjectID(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct GitRefname(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct GitSSHRemote(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct GitTimestamp(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct Html(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct PreciseDateTime(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct Uri(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct X509Certificate(pub String);
}

mod query_dsl {
    use super::types::*;
    cynic::query_dsl!(r#"./../../../schemas/github.graphql"#);
}
