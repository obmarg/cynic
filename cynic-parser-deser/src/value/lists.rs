use std::fmt;

use cynic_parser::{
    ConstValue, Span,
    values::{ConstList, iter::Iter},
};

use super::value::DeserValue;

#[derive(Clone, Copy)]
pub struct List<'a> {
    inner: ListInner<'a>,
}

#[derive(Clone, Copy)]
enum ListInner<'a> {
    Const(ConstList<'a>),
}

impl<'a> List<'a> {
    pub fn is_empty(&self) -> bool {
        match self.inner {
            ListInner::Const(inner) => inner.is_empty(),
        }
    }

    pub fn len(&self) -> usize {
        match self.inner {
            ListInner::Const(inner) => inner.len(),
        }
    }

    pub fn span(&self) -> Option<Span> {
        match self.inner {
            ListInner::Const(inner) => Some(inner.span()),
        }
    }

    pub fn items(&self) -> ListIter<'a> {
        ListIter(match self.inner {
            ListInner::Const(inner) => ListIterInner::Const(inner.items()),
        })
    }

    pub fn get(&self, index: usize) -> Option<DeserValue<'a>> {
        self.items().nth(index)
    }
}

// TODO: Maybe resurrect this
// impl PartialEq for List<'_> {
//     fn eq(&self, other: &Self) -> bool {
//         self.len() == other.len() && self.items().zip(other.items()).all(|(lhs, rhs)| lhs == rhs)
//     }
// }

impl fmt::Debug for List<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.items()).finish()
    }
}

pub struct ListIter<'a>(ListIterInner<'a>);

pub enum ListIterInner<'a> {
    Const(Iter<'a, ConstValue<'a>>),
}

impl<'a> Iterator for ListIter<'a> {
    type Item = DeserValue<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match &mut self.0 {
            ListIterInner::Const(iter) => DeserValue::from_const(iter.next()?),
        })
    }
}

impl<'a> IntoIterator for List<'a> {
    type Item = DeserValue<'a>;

    type IntoIter = ListIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.items()
    }
}

impl<'a> From<ConstList<'a>> for List<'a> {
    fn from(value: ConstList<'a>) -> Self {
        List {
            inner: ListInner::Const(value),
        }
    }
}
