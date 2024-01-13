use crate::ast::{ids::ArgumentId, AstLookup};

use super::{values::ValueReader, AstId, ReadContext};

pub struct Argument<'a>(ReadContext<'a, ArgumentId>);

impl<'a> Argument<'a> {
    pub fn name(&self) -> &str {
        let ast = &self.0.ast;
        ast.lookup(ast.lookup(self.0.id).name)
    }

    pub fn value(&self) -> ValueReader<'a> {
        let ast = &self.0.ast;

        ast.read(ast.lookup(self.0.id).value)
    }
}

impl AstId for ArgumentId {
    type Reader<'a> = Argument<'a>;
}

impl<'a> From<ReadContext<'a, ArgumentId>> for Argument<'a> {
    fn from(value: ReadContext<'a, ArgumentId>) -> Self {
        Self(value)
    }
}
