use crate::{
    AstLookup, Span,
    common::{TypeWrappers, TypeWrappersIter, WrappingType},
    type_system::ids::TypeId,
};

use super::{Definition, ReadContext, StringId, TypeDefinition, TypeSystemId, iter::Iter};

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

    pub fn is_list(&self) -> bool {
        self.wrappers().any(|wrapper| wrapper == WrappingType::List)
    }

    pub fn is_non_null(&self) -> bool {
        self.wrappers().next() == Some(WrappingType::NonNull)
    }

    /// The span of the the type, including any wrapppers
    pub fn span(&self) -> Span {
        self.0.document.lookup(self.0.id).span
    }

    /// The wrapper types from the outermost to innermost
    pub fn wrappers(&self) -> TypeWrappersIter {
        self.0.document.lookup(self.0.id).wrappers.iter()
    }

    /// Returns any definitions of the inner named type
    ///
    /// Note that this iterator scales linearly with the number of types present
    /// in a schema, so should not be used if large schemas are expected.
    pub fn definitions(&self) -> NamedTypeDefinitions<'a> {
        let document = self.0.document;
        NamedTypeDefinitions {
            name: self.name(),
            iter: document.definitions(),
        }
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

impl TypeSystemId for TypeId {
    type Reader<'a> = Type<'a>;

    fn read(self, document: &super::TypeSystemDocument) -> Self::Reader<'_> {
        Type(ReadContext { id: self, document })
    }
}

impl<'a> From<ReadContext<'a, TypeId>> for Type<'a> {
    fn from(value: ReadContext<'a, TypeId>) -> Self {
        Self(value)
    }
}

/// An Iterator over the definitions of a named [Type]
///
/// Note that this is not optimised and scales linearly with the number of definitions in
/// the schema.
#[derive(Clone)]
pub struct NamedTypeDefinitions<'a> {
    name: &'a str,
    iter: Iter<'a, Definition<'a>>,
}

/// A [TypeDefintion] associated with a named [Type]
pub enum NamedTypeDefinition<'a> {
    Definition(TypeDefinition<'a>),
    Extension(TypeDefinition<'a>),
}

impl<'a> Iterator for NamedTypeDefinitions<'a> {
    type Item = NamedTypeDefinition<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next()? {
                Definition::Type(type_definition) if type_definition.name() == self.name => {
                    return Some(NamedTypeDefinition::Definition(type_definition));
                }
                Definition::TypeExtension(type_definition)
                    if type_definition.name() == self.name =>
                {
                    return Some(NamedTypeDefinition::Extension(type_definition));
                }
                _ => continue,
            }
        }
    }
}
