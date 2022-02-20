use std::{borrow::Cow, collections::HashSet};

use once_cell::sync::Lazy;

static RAW_KEYWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    // A list of keywords in rust that can be converted to raw identifiers
    // Taken from https://doc.rust-lang.org/reference/keywords.html
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

    set
});

static NON_RAW_KEYWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    // A list of keywords in rust that cannot be converted to raw identifiers
    // Taken from https://github.com/rust-lang/rust/blob/1.31.1/src/libsyntax_pos/symbol.rs#L456-L460
    let mut set = HashSet::new();

    set.insert("super");
    set.insert("self");
    set.insert("Self");
    set.insert("extern");
    set.insert("crate");

    set
});

pub fn transform_keywords(s: &str) -> Cow<str> {
    let s_ref: &str = s;
    if NON_RAW_KEYWORDS.contains(s_ref) {
        format!("{}_", s).into()
    } else if RAW_KEYWORDS.contains(s_ref) {
        format!("r#{}", s).into()
    } else {
        s.into()
    }
}
