use std::fmt;

use cynic_parser::{
    type_system,
    values::{self, ConstObject, ConstObjectField},
    Span,
};

use super::value::DeserValue;

#[derive(Clone, Copy)]
pub struct Object<'a> {
    inner: ObjectInner<'a>,
}

#[derive(Clone, Copy)]
enum ObjectInner<'a> {
    Const(ConstObject<'a>),
    TypeDirective(type_system::Directive<'a>),
}

impl<'a> Object<'a> {
    pub fn is_empty(&self) -> bool {
        match self.inner {
            ObjectInner::Const(inner) => inner.is_empty(),
            ObjectInner::TypeDirective(directive) => directive.arguments().len() != 0,
        }
    }

    pub fn len(&self) -> usize {
        match self.inner {
            ObjectInner::Const(inner) => inner.len(),
            ObjectInner::TypeDirective(directive) => directive.arguments().len(),
        }
    }

    pub fn span(&self) -> Option<Span> {
        match self.inner {
            ObjectInner::Const(inner) => Some(inner.span()),
            ObjectInner::TypeDirective(directive) => Some(directive.arguments_span()),
        }
    }

    pub fn fields(&self) -> FieldIter<'a> {
        match self.inner {
            ObjectInner::Const(inner) => FieldIter(FieldIterInner::Const(inner.fields())),
            ObjectInner::TypeDirective(inner) => {
                FieldIter(FieldIterInner::TypeDirective(inner.arguments()))
            }
        }
    }

    pub fn get(&self, name: &str) -> Option<DeserValue<'a>> {
        Some(self.fields().find(|field| field.name() == name)?.value())
    }
}

// TODO: Maybe reserrrect this
// impl PartialEq for Object<'_> {
//     fn eq(&self, other: &Self) -> bool {
//         self.len() == other.len()
//             && self.fields().all(|field| {
//                 let needle = field.name();
//                 let Some(b_field) = other
//                     .fields()
//                     .find(|other_field| other_field.name() == needle)
//                 else {
//                     return false;
//                 };

//                 field.value() == b_field.value()
//             })
//     }
// }

impl fmt::Debug for Object<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.fields().map(|field| (field.name(), field.value())))
            .finish()
    }
}

impl<'a> IntoIterator for Object<'a> {
    type Item = ObjectField<'a>;

    type IntoIter = FieldIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields()
    }
}

pub struct FieldIter<'a>(FieldIterInner<'a>);

enum FieldIterInner<'a> {
    Const(values::Iter<'a, cynic_parser::values::ConstObjectField<'a>>),
    TypeDirective(type_system::Iter<'a, type_system::Argument<'a>>),
}

impl<'a> Iterator for FieldIter<'a> {
    type Item = ObjectField<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match &mut self.0 {
            FieldIterInner::Const(iter) => ObjectField {
                inner: FieldInner::Const(iter.next()?),
            },
            FieldIterInner::TypeDirective(iter) => ObjectField {
                inner: FieldInner::TypeDirective(iter.next()?),
            },
        })
    }
}

#[derive(Clone, Copy)]
pub struct ObjectField<'a> {
    inner: FieldInner<'a>,
}

#[derive(Clone, Copy)]
enum FieldInner<'a> {
    Const(ConstObjectField<'a>),
    TypeDirective(type_system::Argument<'a>),
}

impl<'a> ObjectField<'a> {
    pub fn name(&self) -> &'a str {
        match self.inner {
            FieldInner::Const(inner) => inner.name(),
            FieldInner::TypeDirective(inner) => inner.name(),
        }
    }

    pub fn name_span(&self) -> Option<Span> {
        match self.inner {
            FieldInner::Const(inner) => Some(inner.name_span()),
            FieldInner::TypeDirective(inner) => Some(inner.name_span()),
        }
    }

    pub fn value(&self) -> DeserValue<'a> {
        match self.inner {
            FieldInner::Const(inner) => DeserValue::from_const(inner.value()),
            FieldInner::TypeDirective(inner) => DeserValue::from_const(inner.value()),
        }
    }
}
