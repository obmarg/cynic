use crate::{
    AstLookup, Span,
    common::{TypeWrappers, TypeWrappersIter, WrappingType},
};

use super::{
    ExecutableId, ReadContext,
    ids::{StringId, TypeId},
};

pub struct TypeRecord {
    pub name: StringId,
    pub name_start: usize,
    pub wrappers: TypeWrappers,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct Type<'a>(ReadContext<'a, TypeId>);

impl PartialEq for Type<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name() && self.wrappers().eq(other.wrappers())
    }
}

impl Eq for Type<'_> {}

impl<'a> Type<'a> {
    pub fn name(&self) -> &'a str {
        self.0
            .document
            .lookup(self.0.document.lookup(self.0.id).name)
    }

    /// The span of this types named type
    pub fn name_span(&self) -> Span {
        let record = self.0.document.lookup(self.0.id);

        Span::new(
            record.name_start,
            record.name_start + self.0.document.lookup(record.name).len(),
        )
    }

    /// The wrapper types from the outermost to innermost
    pub fn wrappers(&self) -> TypeWrappersIter {
        self.0.document.lookup(self.0.id).wrappers.iter()
    }

    /// The span of the the type, including any wrapppers
    pub fn span(&self) -> Span {
        self.0.document.lookup(self.0.id).span
    }
}

impl std::fmt::Display for Type<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ast = &self.0.document;

        let TypeRecord { name, wrappers, .. } = ast.lookup(self.0.id);

        let wrappers = wrappers.iter().collect::<Vec<_>>();
        for wrapping in &wrappers {
            if let WrappingType::List = wrapping {
                write!(f, "[")?;
            }
        }
        write!(f, "{}", ast.lookup(*name))?;
        for wrapping in wrappers.iter().rev() {
            match wrapping {
                WrappingType::NonNull => write!(f, "!")?,
                WrappingType::List => write!(f, "]")?,
            }
        }

        Ok(())
    }
}

impl std::fmt::Debug for Type<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Type").field(&self.to_string()).finish()
    }
}

impl ExecutableId for TypeId {
    type Reader<'a> = Type<'a>;

    fn read(self, document: &super::ExecutableDocument) -> Self::Reader<'_> {
        Type(ReadContext { id: self, document })
    }
}
