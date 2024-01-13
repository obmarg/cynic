use crate::ast::ids::{ArgumentId, AstLookup};

use super::{values::ValueReader, AstId, AstReader};

pub struct Argument<'a>(AstReader<'a, ArgumentId>);

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

impl<'a> From<AstReader<'a, ArgumentId>> for Argument<'a> {
    fn from(value: AstReader<'a, ArgumentId>) -> Self {
        Self(value)
    }
}
