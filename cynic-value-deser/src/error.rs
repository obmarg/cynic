use std::fmt;

use cynic_parser::Span;

use crate::{value::ValueType, DeserValue};

#[derive(Debug)]
pub enum Error {
    UnexpectedType {
        expected: ValueType,
        found: ValueType,
        span: Option<Span>,
    },
    MissingField {
        name: String,
        object_span: Option<Span>,
    },
    UnknownField {
        name: String,
        field_type: ValueType,
        // TODO: This needs an appropriate span
    },
    Custom {
        text: String,
        span: Option<Span>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnexpectedType {
                expected, found, ..
            } => write!(f, "found a {found} where we expected a {expected}"),
            Error::MissingField { name, object_span } => write!(f, "missing field: {name}"),
            Error::UnknownField { name, field_type } => write!(f, "unknown field: {name}"),
            Error::Custom { text, span } => write!(f, "{text}"),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn unexpected_type(expected: ValueType, found: DeserValue<'_>) -> Self {
        Error::UnexpectedType {
            expected,
            found: ValueType::from(found),
            span: found.span(),
        }
    }

    pub fn custom(text: impl Into<String>, span: Option<Span>) -> Self {
        Error::Custom {
            text: text.into(),
            span,
        }
    }
}
