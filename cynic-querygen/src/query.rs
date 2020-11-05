// Alias all the graphql_parser query types so we don't have to specify generic parameters
// everywhere
pub type Document<'a> = graphql_parser::query::Document<'a, &'a str>;
pub type Definition<'a> = graphql_parser::query::Definition<'a, &'a str>;
pub type FragmentDefinition<'a> = graphql_parser::query::FragmentDefinition<'a, &'a str>;
pub type OperationDefinition<'a> = graphql_parser::query::OperationDefinition<'a, &'a str>;
pub type SelectionSet<'a> = graphql_parser::query::SelectionSet<'a, &'a str>;
pub type Selection<'a> = graphql_parser::query::Selection<'a, &'a str>;
pub type Field<'a> = graphql_parser::query::Field<'a, &'a str>;
pub type FragmentSpread<'a> = graphql_parser::query::FragmentSpread<'a, &'a str>;
pub type InlineFragment<'a> = graphql_parser::query::InlineFragment<'a, &'a str>;
pub type Value<'a> = graphql_parser::query::Value<'a, &'a str>;
pub type VariableDefinition<'a> = graphql_parser::query::VariableDefinition<'a, &'a str>;

pub use graphql_parser::query::ParseError;
