use super::prelude::*;
use super::{
    ids::{BooleanValueId, FloatValueId, IntValueId, NullValueId, StringValueId},
    ReadContext, ValueId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct IntValueRecord {
    pub value: Int,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct IntValue<'a>(pub(in super::super) ReadContext<'a, IntValueId>);

impl<'a> IntValue<'a> {
    pub fn value(&self) -> Int {
        let document = self.0.document;
        document.lookup(self.0.id).value
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl IntValue<'_> {
    pub fn id(&self) -> IntValueId {
        self.0.id
    }
}

impl fmt::Debug for IntValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntValue")
            .field("value", &self.value())
            .field("span", &self.span())
            .finish()
    }
}

impl ValueId for IntValueId {
    type Reader<'a> = IntValue<'a>;
}

impl IdReader for IntValue<'_> {
    type Id = IntValueId;
}

impl<'a> From<ReadContext<'a, IntValueId>> for IntValue<'a> {
    fn from(value: ReadContext<'a, IntValueId>) -> Self {
        Self(value)
    }
}

pub struct FloatValueRecord {
    pub value: Float,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct FloatValue<'a>(pub(in super::super) ReadContext<'a, FloatValueId>);

impl<'a> FloatValue<'a> {
    pub fn value(&self) -> Float {
        let document = self.0.document;
        document.lookup(self.0.id).value
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl FloatValue<'_> {
    pub fn id(&self) -> FloatValueId {
        self.0.id
    }
}

impl fmt::Debug for FloatValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FloatValue")
            .field("value", &self.value())
            .field("span", &self.span())
            .finish()
    }
}

impl ValueId for FloatValueId {
    type Reader<'a> = FloatValue<'a>;
}

impl IdReader for FloatValue<'_> {
    type Id = FloatValueId;
}

impl<'a> From<ReadContext<'a, FloatValueId>> for FloatValue<'a> {
    fn from(value: ReadContext<'a, FloatValueId>) -> Self {
        Self(value)
    }
}

pub struct StringValueRecord {
    pub value: StringId,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct StringValue<'a>(pub(in super::super) ReadContext<'a, StringValueId>);

impl<'a> StringValue<'a> {
    pub fn value(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).value)
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl StringValue<'_> {
    pub fn id(&self) -> StringValueId {
        self.0.id
    }
}

impl fmt::Debug for StringValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StringValue")
            .field("value", &self.value())
            .field("span", &self.span())
            .finish()
    }
}

impl ValueId for StringValueId {
    type Reader<'a> = StringValue<'a>;
}

impl IdReader for StringValue<'_> {
    type Id = StringValueId;
}

impl<'a> From<ReadContext<'a, StringValueId>> for StringValue<'a> {
    fn from(value: ReadContext<'a, StringValueId>) -> Self {
        Self(value)
    }
}

pub struct BooleanValueRecord {
    pub value: bool,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct BooleanValue<'a>(pub(in super::super) ReadContext<'a, BooleanValueId>);

impl<'a> BooleanValue<'a> {
    pub fn value(&self) -> bool {
        let document = self.0.document;
        document.lookup(self.0.id).value
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl BooleanValue<'_> {
    pub fn id(&self) -> BooleanValueId {
        self.0.id
    }
}

impl fmt::Debug for BooleanValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BooleanValue")
            .field("value", &self.value())
            .field("span", &self.span())
            .finish()
    }
}

impl ValueId for BooleanValueId {
    type Reader<'a> = BooleanValue<'a>;
}

impl IdReader for BooleanValue<'_> {
    type Id = BooleanValueId;
}

impl<'a> From<ReadContext<'a, BooleanValueId>> for BooleanValue<'a> {
    fn from(value: ReadContext<'a, BooleanValueId>) -> Self {
        Self(value)
    }
}

pub struct NullValueRecord {
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct NullValue<'a>(pub(in super::super) ReadContext<'a, NullValueId>);

impl<'a> NullValue<'a> {
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl NullValue<'_> {
    pub fn id(&self) -> NullValueId {
        self.0.id
    }
}

impl fmt::Debug for NullValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NullValue")
            .field("span", &self.span())
            .finish()
    }
}

impl ValueId for NullValueId {
    type Reader<'a> = NullValue<'a>;
}

impl IdReader for NullValue<'_> {
    type Id = NullValueId;
}

impl<'a> From<ReadContext<'a, NullValueId>> for NullValue<'a> {
    fn from(value: ReadContext<'a, NullValueId>) -> Self {
        Self(value)
    }
}