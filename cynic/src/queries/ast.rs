use std::{borrow::Cow, fmt::Write};

use super::indent::indented;

#[derive(Debug, Default)]
/// A set of field selections that form part of a graphql query.
pub struct SelectionSet {
    pub(super) selections: Vec<Selection>,
}

#[derive(Debug)]
/// An individual selection
pub enum Selection {
    /// Selects a field
    Field(FieldSelection),
    /// Selects an inline fragment
    InlineFragment(InlineFragment),
}

#[derive(Debug)]
/// The details of a particular field selection
pub struct FieldSelection {
    pub(super) name: &'static str,
    pub(super) alias: Option<Cow<'static, str>>,
    pub(super) arguments: Vec<Argument>,
    pub(super) children: SelectionSet,
}

#[derive(Debug, PartialEq)]
/// An argument
pub struct Argument {
    pub(super) name: Cow<'static, str>,
    pub(super) value: InputLiteral,
}

impl Argument {
    /// Constructs an `Argument`
    pub fn new(name: &'static str, value: InputLiteral) -> Self {
        Argument {
            name: Cow::Borrowed(name),
            value,
        }
    }

    /// Constructs an `Argument` with a `Cow` as its name
    pub fn from_cow_name(name: Cow<'static, str>, value: InputLiteral) -> Self {
        Argument { name, value }
    }
}

#[derive(Debug, PartialEq)]
/// An `InputLiteral` is an argument that will be output in the GraphQL
/// query text (as opposed to a variable that will go in the variables
/// field)
pub enum InputLiteral {
    /// An integer
    Int(i32),
    /// A float
    Float(f64),
    /// A boolean
    Bool(bool),
    /// A string
    String(Cow<'static, str>),
    /// An ID
    Id(String),
    /// An object
    Object(Vec<Argument>),
    /// A list
    List(Vec<InputLiteral>),
    /// A variable
    Variable(&'static str),
    /// A null
    Null,
    /// One of the values of an enum
    EnumValue(&'static str),
}

#[derive(Debug, Default)]
/// An inline fragment that selects fields from one possible type
pub struct InlineFragment {
    pub(super) on_clause: Option<&'static str>,
    pub(super) children: SelectionSet,
}

impl FieldSelection {
    /// Creates a new FieldSelection
    pub fn new(name: &'static str) -> FieldSelection {
        FieldSelection {
            name,
            alias: None,
            arguments: Vec::new(),
            children: SelectionSet::default(),
        }
    }
}

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
                if let Some(alias) = &field_selection.alias {
                    write!(f, "{}: ", alias)?;
                }

                write!(f, "{}", field_selection.name)?;

                if !field_selection.arguments.is_empty() {
                    write!(f, "(")?;
                    let mut first = true;
                    for arg in &field_selection.arguments {
                        if !first {
                            write!(f, ", ")?;
                        }
                        first = false;
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ")")?;
                }
                write!(f, "{}", field_selection.children)
            }
            Selection::InlineFragment(inline_fragment) => {
                write!(f, "...")?;
                if let Some(on_type) = inline_fragment.on_clause {
                    write!(f, " on {}", on_type)?;
                }
                write!(f, "{}", inline_fragment.children)
            }
        }
    }
}

impl std::fmt::Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

impl std::fmt::Display for InputLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputLiteral::Int(val) => write!(f, "{}", val),
            InputLiteral::Float(val) => write!(f, "{}", val),
            InputLiteral::Bool(val) => write!(f, "{}", val),
            InputLiteral::String(val) => write!(f, "\"{}\"", val),
            InputLiteral::Id(val) => write!(f, "\"{}\"", val),
            InputLiteral::Object(fields) => {
                write!(f, "{{")?;
                for field in fields {
                    write!(f, "{}: {}, ", field.name, field.value)?;
                }
                write!(f, "}}")
            }
            InputLiteral::List(vals) => {
                write!(f, "[")?;
                for val in vals {
                    write!(f, "{}, ", val)?;
                }
                write!(f, "]")
            }
            InputLiteral::Variable(name) => {
                write!(f, "${}", name)
            }
            InputLiteral::Null => {
                write!(f, "null")
            }
            InputLiteral::EnumValue(name) => {
                write!(f, "{name}")
            }
        }
    }
}
