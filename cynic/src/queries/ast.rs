#![allow(dead_code, unused_variables, missing_docs)]
// TODO: Don't allow the above

use std::fmt::Write;

use crate::indent::indented;

#[derive(Debug, Default)]
pub struct SelectionSet {
    pub selections: Vec<Selection>,
}

#[derive(Debug)]
pub enum Selection {
    Field(FieldSelection),
    InlineFragment(InlineFragment),
    FragmentSpread(FragmentSpread),
}

#[derive(Debug)]
pub struct FieldSelection {
    pub name: &'static str,
    pub children: SelectionSet,
}

#[derive(Debug, Default)]
pub struct InlineFragment {
    pub on_clause: Option<&'static str>,
    pub children: SelectionSet,
}

#[derive(Debug)]
pub struct FragmentSpread {}

impl std::fmt::Display for SelectionSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.selections.is_empty() {
            writeln!(f, " {{")?;
            for child in &self.selections {
                write!(indented(f, 2), "{}", child)?;
            }
            write!(f, "}}")?;
        }
        writeln!(f)
    }
}

impl std::fmt::Display for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Selection::Field(field_selection) => {
                write!(f, "{}", field_selection.name)?;
                write!(f, "{}", field_selection.children)
            }
            Selection::InlineFragment(inline_fragment) => {
                write!(f, "...")?;
                if let Some(on_type) = inline_fragment.on_clause {
                    write!(f, " on {}", on_type)?;
                }
                write!(f, "{}", inline_fragment.children)
            }
            Selection::FragmentSpread(_) => todo!(),
        }
    }
}
