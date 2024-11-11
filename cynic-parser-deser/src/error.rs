use std::fmt;

use cynic_parser::Span;

use crate::{value::ValueType, DeserValue};

// TODO: Should these errors have paths in them as well?

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
    DuplicateField {
        name: String,
        // TODO: This needs two spans: the original field
        // and the duplicate field
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
            Error::MissingField { name, .. } => write!(f, "missing field: {name}"),
            Error::UnknownField { name, .. } => write!(f, "unknown field: {name}"),
            Error::DuplicateField { name } => write!(f, "duplicate field: {name}"),
            Error::Custom { text, .. } => write!(f, "{text}"),
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
