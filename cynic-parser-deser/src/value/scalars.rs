use core::fmt;

use cynic_parser::Span;

#[derive(Clone, Copy)]
pub struct IntValue {
    pub(crate) value: i64,
    pub(crate) span: Option<Span>,
}

impl IntValue {
    pub fn as_i64(&self) -> i64 {
        self.value
    }

    pub fn as_i32(&self) -> i32 {
        self.value as i32
    }

    fn value(&self) -> i64 {
        self.value
    }

    pub fn span(&self) -> Option<Span> {
        self.span
    }
}

impl PartialEq for IntValue {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl Eq for IntValue {}

impl std::fmt::Debug for IntValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl std::fmt::Display for IntValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[derive(Clone, Copy)]
pub struct FloatValue {
    pub(crate) value: f64,
    pub(crate) span: Option<Span>,
}

impl FloatValue {
    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn as_f64(&self) -> f64 {
        self.value()
    }

    pub fn span(&self) -> Option<Span> {
        self.span
    }
}

impl PartialEq for FloatValue {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl fmt::Debug for FloatValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl fmt::Display for FloatValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[derive(Clone, Copy)]
pub struct StringValue<'a> {
    pub(crate) value: &'a str,
    pub(crate) span: Option<Span>,
}

impl<'a> StringValue<'a> {
    pub fn value(&self) -> &'a str {
        self.value
    }

    pub fn as_str(&self) -> &'a str {
        self.value()
    }

    pub fn span(&self) -> Option<Span> {
        self.span
    }
}

impl PartialEq for StringValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl Eq for StringValue<'_> {}

impl fmt::Debug for StringValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl fmt::Display for StringValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[derive(Clone, Copy)]
pub struct BooleanValue {
    pub(crate) value: bool,
    pub(crate) span: Option<Span>,
}

impl BooleanValue {
    pub fn value(&self) -> bool {
        self.value
    }

    pub fn as_bool(&self) -> bool {
        self.value()
    }

    pub fn span(&self) -> Option<Span> {
        self.span
    }
}

impl PartialEq for BooleanValue {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl Eq for BooleanValue {}

impl fmt::Debug for BooleanValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl fmt::Display for BooleanValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[derive(Clone, Copy)]
pub struct NullValue {
    pub(crate) span: Option<Span>,
}

impl NullValue {
    pub fn span(&self) -> Option<Span> {
        self.span
    }
}

impl PartialEq for NullValue {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for NullValue {}

impl fmt::Debug for NullValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "null")
    }
}

impl fmt::Display for NullValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "null")
    }
}
