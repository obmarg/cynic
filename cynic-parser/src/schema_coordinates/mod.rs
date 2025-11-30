use crate::{Error, Span};

mod lexer;
mod ty;

mod argument;
mod directive;
mod directive_argument;
mod member;
#[allow(unused_braces)]
mod parser;

pub use self::{
    argument::ArgumentCoordinate, directive::DirectiveCoordinate,
    directive_argument::DirectiveArgumentCoordinate, member::MemberCoordinate, ty::TypeCoordinate,
};

pub fn parse_schema_coordinate(input: &str) -> Result<SchemaCoordinate, Error> {
    if input.trim().is_empty() {
        return Err(Error::EmptySchemaCoordinate);
    }

    let lexer = lexer::Lexer::new(input);

    Ok(parser::SchemaCoordinateParser::new().parse(input, lexer)?)
}

/// A GraphQL [Schema Coordinate][1]
///
/// [1]: https://spec.graphql.org/September2025/#sec-Schema-Coordinates
pub enum SchemaCoordinate {
    Type(TypeCoordinate),
    Member(MemberCoordinate),
    Argument(ArgumentCoordinate),
    Directive(DirectiveCoordinate),
    DirectiveArgument(DirectiveArgumentCoordinate),
}

pub struct Name {
    span: Span,
    value: Box<str>,
}
