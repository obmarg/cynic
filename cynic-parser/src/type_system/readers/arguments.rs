use crate::type_system::ids::ArgumentId;
use crate::AstLookup;

use super::{values::ValueReader, ReadContext, TypeSystemId};

#[derive(Clone, Copy)]
pub struct Argument<'a>(ReadContext<'a, ArgumentId>);

impl<'a> Argument<'a> {
    pub fn name(&self) -> &'a str {
        let ast = &self.0.ast;
        ast.lookup(ast.lookup(self.0.id).name)
    }

    pub fn value(&self) -> ValueReader<'a> {
        let ast = &self.0.ast;

        ast.read(ast.lookup(self.0.id).value)
    }
}

impl TypeSystemId for ArgumentId {
    type Reader<'a> = Argument<'a>;
}

impl<'a> From<ReadContext<'a, ArgumentId>> for Argument<'a> {
    fn from(value: ReadContext<'a, ArgumentId>) -> Self {
        Self(value)
    }
}
