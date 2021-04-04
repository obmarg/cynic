#![allow(unused_imports)]

fn main() {}

#[cynic::schema_for_derives(file = r#"./../../../schemas/github.graphql"#, module = "schema")]
mod queries {
    use super::{schema, types::*};

    #[derive(cynic::Enum, Clone, Copy, Debug)]
    pub enum IssueOrderField {
        Comments,
        CreatedAt,
        UpdatedAt,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug)]
    pub enum OrderDirection {
        Asc,
        Desc,
    }

    #[derive(cynic::InputObject, Debug)]
    pub struct IssueOrder {
        pub direction: OrderDirection,
        pub fieid: IssueOrderField,
    }
}

#[cynic::schema_for_derives(file = r#"./../../../schemas/github.graphql"#, module = "schema")]
mod types {
    use super::schema;

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

mod schema {
    use super::types::*;
    cynic::use_schema!(r#"./../../../schemas/github.graphql"#);
}
