use std::{borrow::Cow, collections::HashSet};

use once_cell::sync::Lazy;

use crate::schema::TypeSpec;

pub struct Field<'a> {
    name: &'a str,
    rename: Option<&'a str>,
    type_spec: &'a TypeSpec<'a>,
}

impl<'a> Field<'a> {
    pub fn new(name: &'a str, type_spec: &'a TypeSpec<'a>) -> Self {
        Field {
            name,
            type_spec,
            rename: None,
        }
    }

    pub fn add_rename(&mut self, name: &'a str) {
        self.rename = Some(name);
    }

    fn name(&self) -> Cow<'a, str> {
        rust_field_name(self.name)
    }

    fn rename(&self) -> Option<&'a str> {
        if let Some(rename) = self.rename {
            return Some(rename);
        }
        if KEYWORDS.contains(self.name) || self.name == "_" {
            return Some(self.name);
        }

        None
    }
}

impl std::fmt::Display for Field<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(rename) = self.rename() {
            writeln!(f, r#"#[cynic(rename = "{}")]"#, rename)?;
        }
        if self.type_spec.name.starts_with("cynic::MaybeUndefined<") {
            writeln!(
                f,
                r#"#[cynic(skip_serializing_if = "cynic::MaybeUndefined::is_undefined")]"#
            )?;
        }
        writeln!(f, "pub {}: {},", self.name(), self.type_spec.name)
    }
}

pub fn rust_field_name(name: &str) -> Cow<'_, str> {
    if KEYWORDS.contains(name) {
        return Cow::Owned(format!("{}_", name));
    }

    if name == "_" {
        return Cow::Borrowed("__underscore");
    }

    Cow::Borrowed(name)
}

// A list of keywords in rust that can be converted to raw identifiers
// Taken from https://doc.rust-lang.org/reference/keywords.html
static KEYWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();

    // Strict Keywords 2015
    set.insert("as");
    set.insert("break");
    set.insert("const");
    set.insert("continue");
    set.insert("else");
    set.insert("enum");
    set.insert("false");
    set.insert("fn");
    set.insert("for");
    set.insert("if");
    set.insert("impl");
    set.insert("in");
    set.insert("let");
    set.insert("loop");
    set.insert("match");
    set.insert("mod");
    set.insert("move");
    set.insert("mut");
    set.insert("pub");
    set.insert("ref");
    set.insert("return");
    set.insert("static");
    set.insert("struct");
    set.insert("trait");
    set.insert("true");
    set.insert("type");
    set.insert("unsafe");
    set.insert("use");
    set.insert("where");
    set.insert("while");

    // Strict keywords 2018
    set.insert("async");
    set.insert("await");
    set.insert("dyn");

    // Reserved Keywords 2015
    set.insert("abstract");
    set.insert("become");
    set.insert("box");
    set.insert("do");
    set.insert("final");
    set.insert("macro");
    set.insert("override");
    set.insert("priv");
    set.insert("typeof");
    set.insert("unsized");
    set.insert("virtual");
    set.insert("yield");

    // Reserved Keywords 2018
    set.insert("try");

    // Additional keywords that can't be used in raw idents
    set.insert("super");
    set.insert("self");
    set.insert("Self");
    set.insert("extern");
    set.insert("crate");

    set
});
