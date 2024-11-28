use core::fmt;

use crate::{AstLookup, Span};

use super::{ids::ValueId, Cursor};

#[derive(Clone, Copy)]
pub struct IntValue<'a>(pub(super) Cursor<'a, ValueId>);

impl IntValue<'_> {
    pub fn as_i64(&self) -> i64 {
        self.value()
    }

    pub fn as_i32(&self) -> i32 {
        self.value() as i32
    }

    pub fn value(&self) -> i64 {
        let store = self.0.store;
        store.lookup(self.0.id).kind.as_int().unwrap()
    }

    pub fn span(&self) -> Span {
        let store = self.0.store;
        store.lookup(self.0.id).span
    }
}

impl IntValue<'_> {
    pub fn id(&self) -> ValueId {
        self.0.id
    }
}

impl PartialEq for IntValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl Eq for IntValue<'_> {}

impl std::fmt::Debug for IntValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl std::fmt::Display for IntValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[derive(Clone, Copy)]
pub struct FloatValue<'a>(pub(super) Cursor<'a, ValueId>);

impl FloatValue<'_> {
    pub fn value(&self) -> f64 {
        let store = self.0.store;
        store.lookup(self.0.id).kind.as_float().unwrap()
    }

    pub fn as_f64(&self) -> f64 {
        self.value()
    }

    pub fn span(&self) -> Span {
        let store = self.0.store;
        store.lookup(self.0.id).span
    }
}

impl FloatValue<'_> {
    pub fn id(&self) -> ValueId {
        self.0.id
    }
}

impl PartialEq for FloatValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl fmt::Debug for FloatValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl fmt::Display for FloatValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[derive(Clone, Copy)]
pub struct StringValue<'a>(pub(super) Cursor<'a, ValueId>);

impl<'a> StringValue<'a> {
    pub fn value(&self) -> &'a str {
        let store = &self.0.store;
        store.lookup(store.lookup(self.0.id).kind.as_string().unwrap())
    }

    pub fn as_str(&self) -> &'a str {
        self.value()
    }

    pub fn span(&self) -> Span {
        let store = self.0.store;
        store.lookup(self.0.id).span
    }
}

impl StringValue<'_> {
    pub fn id(&self) -> ValueId {
        self.0.id
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
pub struct BooleanValue<'a>(pub(super) Cursor<'a, ValueId>);

impl BooleanValue<'_> {
    pub fn value(&self) -> bool {
        let store = self.0.store;
        store.lookup(self.0.id).kind.as_boolean().unwrap()
    }

    pub fn as_bool(&self) -> bool {
        self.value()
    }

    pub fn span(&self) -> Span {
        let store = self.0.store;
        store.lookup(self.0.id).span
    }
}

impl BooleanValue<'_> {
    pub fn id(&self) -> ValueId {
        self.0.id
    }
}

impl PartialEq for BooleanValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl Eq for BooleanValue<'_> {}

impl fmt::Debug for BooleanValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl fmt::Display for BooleanValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[derive(Clone, Copy)]
pub struct NullValue<'a>(pub(super) Cursor<'a, ValueId>);

impl NullValue<'_> {
    pub fn span(&self) -> Span {
        let store = self.0.store;
        store.lookup(self.0.id).span
    }
}

impl NullValue<'_> {
    pub fn id(&self) -> ValueId {
        self.0.id
    }
}

impl PartialEq for NullValue<'_> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for NullValue<'_> {}

impl fmt::Debug for NullValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "null")
    }
}

impl fmt::Display for NullValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "null")
    }
}
