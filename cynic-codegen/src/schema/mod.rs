mod parser;
mod type_index;
mod types;

pub use self::parser::{load_schema, Document};

// TODO: Remove these once we've stopped using the parser types directly.
pub use self::{parser::*, type_index::TypeIndex};

// TODO: Uncomment this
// pub use self::{types::*},

// TODO: don't allow dead_code

#[allow(dead_code)]
pub struct Schema<'a> {
    doc: &'a Document,
    type_index: type_index::TypeIndex<'a>,
}

#[allow(dead_code)]
impl<'a> Schema<'a> {
    pub fn new(document: &'a Document) -> Self {
        #[allow(dead_code)]
        let type_index = type_index::TypeIndex::for_schema(document);

        Schema {
            doc: document,
            type_index,
        }
    }

    pub fn lookup(&self, name: &str) -> Result<types::Type, SchemaError> {
        Ok(self.type_index.lookup_valid_type(name).expect("TODO"))
    }
}

#[allow(dead_code)]
pub enum SchemaError {
    // TODO: some actual errors
}
