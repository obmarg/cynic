use std::borrow::Cow;

use crate::{common::trim_block_string_whitespace, type_system::StringLiteralId, AstLookup};

use super::{ReadContext, TypeSystemId};

#[derive(Clone, Copy, Debug)]
pub struct StringLiteral<'a>(StringLiteralInner<'a>);

#[derive(Clone, Copy, Debug)]
enum StringLiteralInner<'a> {
    String(&'a str),
    BlockString(&'a str),
}

#[derive(Clone, Copy, Debug)]
pub enum StringLiteralKind {
    String,
    Block,
}

impl<'a> StringLiteral<'a> {
    pub fn to_cow(&self) -> Cow<'a, str> {
        match self.0 {
            StringLiteralInner::String(string) => Cow::Borrowed(string),
            StringLiteralInner::BlockString(string) => {
                // This is more intense - we need to unquote the string
                //
                // TODO: Look into recording what work needs done at parse time
                Cow::Owned(trim_block_string_whitespace(string))
            }
        }
    }

    pub fn raw_str(&self) -> &'a str {
        match self.0 {
            StringLiteralInner::String(string) => string,
            StringLiteralInner::BlockString(string) => string,
        }
    }

    pub fn kind(&self) -> StringLiteralKind {
        match self.0 {
            StringLiteralInner::String(_) => StringLiteralKind::String,
            StringLiteralInner::BlockString(_) => StringLiteralKind::Block,
        }
    }
}

impl TypeSystemId for StringLiteralId {
    type Reader<'a> = StringLiteral<'a>;
}

impl<'a> From<ReadContext<'a, StringLiteralId>> for StringLiteral<'a> {
    fn from(value: ReadContext<'a, StringLiteralId>) -> Self {
        StringLiteral(match value.id {
            StringLiteralId::String(id) => StringLiteralInner::String(value.document.lookup(id)),
            StringLiteralId::Block(id) => {
                StringLiteralInner::BlockString(value.document.lookup(id))
            }
        })
    }
}
