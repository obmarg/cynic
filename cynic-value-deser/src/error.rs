use cynic_parser::Span;

use crate::{value::ValueType, DeserValue};

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
