// Alias all the graphql_parser schema types so we don't have to specify generic parameters
// everywhere
pub type Document<'a> = graphql_parser::schema::Document<'a, &'a str>;
pub type Type<'a> = graphql_parser::schema::Type<'a, &'a str>;
pub type Field<'a> = graphql_parser::schema::Field<'a, &'a str>;
pub type Definition<'a> = graphql_parser::schema::Definition<'a, &'a str>;
pub type TypeDefinition<'a> = graphql_parser::schema::TypeDefinition<'a, &'a str>;
pub type ScalarType<'a> = graphql_parser::schema::ScalarType<'a, &'a str>;
pub type InputValue<'a> = graphql_parser::schema::InputValue<'a, &'a str>;

pub trait ScalarTypeExt {
    fn is_builtin(&self) -> bool;
}

impl<'a> ScalarTypeExt for ScalarType<'a> {
    fn is_builtin(&self) -> bool {
        matches!(self.name, "String" | "Int" | "Boolean" | "ID")
    }
}
