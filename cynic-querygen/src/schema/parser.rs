// Alias all the graphql_parser schema types so we don't have to specify generic parameters
// everywhere
pub type Document<'a> = graphql_parser::schema::Document<'a, &'a str>;
pub type Type<'a> = graphql_parser::schema::Type<'a, &'a str>;
pub type Field<'a> = graphql_parser::schema::Field<'a, &'a str>;
pub type TypeDefinition<'a> = graphql_parser::schema::TypeDefinition<'a, &'a str>;
pub type InputValue<'a> = graphql_parser::schema::InputValue<'a, &'a str>;

pub(super) fn typename_field<'a>() -> Field<'a> {
    Field {
        position: graphql_parser::Pos { line: 0, column: 0 },
        description: None,
        name: "__typename",
        arguments: vec![],
        field_type: Type::NonNullType(Box::new(Type::NamedType("String"))),
        directives: vec![],
    }
}
