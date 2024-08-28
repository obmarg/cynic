/// Cache and export large github schema, as it is expensive to recreate.
#[cynic::schema("github")]
pub mod github { }