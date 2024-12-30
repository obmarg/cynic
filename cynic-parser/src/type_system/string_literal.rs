use std::{borrow::Cow, fmt};

use crate::{common::trim_block_string_whitespace, type_system::StringLiteralId, AstLookup};

use super::TypeSystemId;

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

    pub fn raw_untrimmed_str(&self) -> &'a str {
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

impl fmt::Display for StringLiteral<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_cow().as_ref())
    }
}

impl TypeSystemId for StringLiteralId {
    type Reader<'a> = StringLiteral<'a>;

    fn read(self, document: &super::TypeSystemDocument) -> Self::Reader<'_> {
        StringLiteral(match self {
            StringLiteralId::String(id) => StringLiteralInner::String(document.lookup(id)),
            StringLiteralId::Block(id) => StringLiteralInner::BlockString(document.lookup(id)),
        })
    }
}
