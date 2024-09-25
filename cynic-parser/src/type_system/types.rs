use crate::{
    common::{TypeWrappers, WrappingType},
    type_system::ids::TypeId,
    AstLookup, Span,
};

use super::{ReadContext, StringId, TypeSystemId};

pub struct TypeRecord {
    pub name: StringId,
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

    pub fn is_list(&self) -> bool {
        self.wrappers().any(|wrapper| wrapper == WrappingType::List)
    }

    pub fn is_non_null(&self) -> bool {
        self.wrappers().next() == Some(WrappingType::NonNull)
    }

    pub fn span(&self) -> Span {
        self.0.document.lookup(self.0.id).span
    }

    /// The wrapper types from the outermost to innermost
    pub fn wrappers(&self) -> impl Iterator<Item = WrappingType> + 'a {
        self.0.document.lookup(self.0.id).wrappers.iter()
    }
}

impl std::fmt::Display for Type<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ast = &self.0.document;

        let TypeRecord {
            name,
            wrappers,
            span: _,
        } = ast.lookup(self.0.id);

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

impl TypeSystemId for TypeId {
    type Reader<'a> = Type<'a>;
}

impl<'a> From<ReadContext<'a, TypeId>> for Type<'a> {
    fn from(value: ReadContext<'a, TypeId>) -> Self {
        Self(value)
    }
}
