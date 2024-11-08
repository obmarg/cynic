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
        expected: ValueType,
        object_span: Option<Span>,
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
