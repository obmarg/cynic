use std::fmt;

use crate::{ExecutableDocument, TypeSystemDocument};

// TODO: Document me?
pub struct PrettyPrinter<'a> {
    document: Document<'a>,
    options: PrettyOptions,
}

#[derive(Default, Clone, Copy)]
pub(super) struct PrettyOptions {
    pub(super) sort: bool,
}

impl<'a> PrettyPrinter<'a> {
    pub(super) fn new_type_system(document: &'a TypeSystemDocument) -> Self {
        PrettyPrinter {
            document: Document::TypeSystem(document),
            options: Default::default(),
        }
    }

    pub(super) fn new_executable(document: &'a ExecutableDocument) -> Self {
        PrettyPrinter {
            document: Document::Executable(document),
            options: Default::default(),
        }
    }

    pub fn sorted(mut self) -> Self {
        self.options.sort = true;
        self
    }
}

impl fmt::Display for PrettyPrinter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.document {
            Document::Executable(doc) => write!(f, "{}", doc.pretty_print(&self.options)),
            Document::TypeSystem(doc) => write!(f, "{}", doc.pretty_print(&self.options)),
        }
    }
}

enum Document<'a> {
    Executable(&'a ExecutableDocument),
    TypeSystem(&'a TypeSystemDocument),
}
