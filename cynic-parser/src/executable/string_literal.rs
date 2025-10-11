use crate::{
    AstLookup,
    common::StringLiteral,
    executable::{ExecutableId, ids::StringLiteralId},
};

impl ExecutableId for StringLiteralId {
    type Reader<'a> = StringLiteral<'a>;

    fn read(self, document: &super::ExecutableDocument) -> Self::Reader<'_> {
        match self {
            StringLiteralId::String(id) => StringLiteral::new_string(document.lookup(id)),
            StringLiteralId::Block(id) => StringLiteral::new_block(document.lookup(id)),
        }
    }
}
