#[derive(Debug)]
pub struct Schema {
    pub query: String,
}

#[derive(Debug)]
pub struct Object {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub ty: String,
}

// Ok, so I _could_
// - Create a Store
// - has fn add_string(&self, str, span) -> NodeId
// - has fn add_definition(&self, blah) -> NodeId
// AST structs mostly just contain NodeIds.
// Spans are associated with NodeIds.
// But NodeIds _also_ contain some kind of NodeKind { Definition(DefId), Blah(Blah) } type
//
// Walker-like types on top of NodeIds allow for easy access to the actual data, spans etc. etc.
