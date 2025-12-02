use std::fmt;

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
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum SchemaCoordinate {
    Type(TypeCoordinate),
    Member(MemberCoordinate),
    Argument(ArgumentCoordinate),
    Directive(DirectiveCoordinate),
    DirectiveArgument(DirectiveArgumentCoordinate),
}

impl SchemaCoordinate {
    pub fn ty(name: impl Into<String>) -> Self {
        SchemaCoordinate::Type(TypeCoordinate::new(name))
    }

    pub fn member(ty: impl Into<String>, field: impl Into<String>) -> Self {
        SchemaCoordinate::Member(MemberCoordinate::new(ty, field))
    }

    pub fn argument(
        ty: impl Into<String>,
        field: impl Into<String>,
        argument: impl Into<String>,
    ) -> Self {
        SchemaCoordinate::Argument(ArgumentCoordinate::new(ty, field, argument))
    }

    pub fn directive(name: impl Into<String>) -> Self {
        SchemaCoordinate::Directive(DirectiveCoordinate::new(name))
    }

    pub fn directive_argument(name: impl Into<String>, argument: impl Into<String>) -> Self {
        SchemaCoordinate::DirectiveArgument(DirectiveArgumentCoordinate::new(name, argument))
    }

    pub fn as_ty(&self) -> Option<&TypeCoordinate> {
        match self {
            SchemaCoordinate::Type(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_member(&self) -> Option<&MemberCoordinate> {
        match self {
            SchemaCoordinate::Member(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_argument(&self) -> Option<&ArgumentCoordinate> {
        match self {
            SchemaCoordinate::Argument(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_directive(&self) -> Option<&DirectiveCoordinate> {
        match self {
            SchemaCoordinate::Directive(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_directive_argument(&self) -> Option<&DirectiveArgumentCoordinate> {
        match self {
            SchemaCoordinate::DirectiveArgument(inner) => Some(inner),
            _ => None,
        }
    }
}

impl fmt::Display for SchemaCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemaCoordinate::Type(inner) => write!(f, "{inner}"),
            SchemaCoordinate::Member(inner) => write!(f, "{inner}"),
            SchemaCoordinate::Argument(inner) => write!(f, "{inner}"),
            SchemaCoordinate::Directive(inner) => write!(f, "{inner}"),
            SchemaCoordinate::DirectiveArgument(inner) => write!(f, "{inner}"),
        }
    }
}

#[derive(Clone, Debug, Eq)]
pub struct Name {
    span: Span,
    value: Box<str>,
}

impl Name {
    fn new(value: String) -> Self {
        Name {
            span: Span::default(),
            value: value.into(),
        }
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for Name {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Name {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl std::hash::Hash for Name {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}
