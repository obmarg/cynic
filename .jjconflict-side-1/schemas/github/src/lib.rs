cynic::use_schema!("../github.graphql");

cynic::impl_scalar!(chrono::DateTime<chrono::Utc>, DateTime);
