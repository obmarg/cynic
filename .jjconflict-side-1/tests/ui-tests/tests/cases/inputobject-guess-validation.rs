#![allow(unused_imports)]

fn main() {}

#[cynic::schema_for_derives(file = r#"./../../../../schemas/github.graphql"#, module = "schema")]
mod queries {
    use super::schema;

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

mod schema {
    cynic::use_schema!(r#"./../../../../schemas/github.graphql"#);
}
