use crate::ast::{
    ids::{AstLookup, TypeId},
    WrappingType,
};

use super::{AstId, ReadContext};

pub struct Type<'a>(ReadContext<'a, TypeId>);

impl<'a> std::fmt::Display for Type<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ast = &self.0.ast;

        let crate::ast::Type { name, wrappers } = ast.lookup(self.0.id);

        for wrapping in wrappers.iter().rev() {
            if let WrappingType::List = wrapping {
                write!(f, "[")?;
            }
        }
        write!(f, "{}", ast.lookup(*name))?;
        for wrapping in wrappers.iter() {
            match wrapping {
                WrappingType::NonNull => write!(f, "!")?,
                WrappingType::List => write!(f, "]")?,
            }
        }

        Ok(())
    }
}

impl AstId for TypeId {
    type Reader<'a> = Type<'a>;
}

impl<'a> From<ReadContext<'a, TypeId>> for Type<'a> {
    fn from(value: ReadContext<'a, TypeId>) -> Self {
        Self(value)
    }
}
