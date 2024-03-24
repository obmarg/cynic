use crate::{
    common::{TypeWrappers, WrappingType},
    type_system::ids::TypeId,
    AstLookup,
};

use super::{ReadContext, StringId, TypeSystemId};

pub struct TypeRecord {
    pub name: StringId,
    pub wrappers: TypeWrappers,
}

#[derive(Clone, Copy)]
pub struct Type<'a>(ReadContext<'a, TypeId>);

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

    /// The wrapper types from the outermost to innermost
    pub fn wrappers(&self) -> impl Iterator<Item = WrappingType> + 'a {
        self.0.document.lookup(self.0.id).wrappers.iter()
    }
}

impl std::fmt::Display for Type<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ast = &self.0.document;

        let TypeRecord { name, wrappers } = ast.lookup(self.0.id);

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

impl TypeSystemId for TypeId {
    type Reader<'a> = Type<'a>;
}

impl<'a> From<ReadContext<'a, TypeId>> for Type<'a> {
    fn from(value: ReadContext<'a, TypeId>) -> Self {
        Self(value)
    }
}
