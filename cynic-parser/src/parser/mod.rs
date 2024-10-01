// These are generated by LARLPOP
mod executable;
mod schema;

pub use executable::*;
pub use schema::*;

use crate::{common::MalformedStringError, lexer::LexicalError};

pub enum AdditionalErrors {
    Lexical(LexicalError),
    MalformedString(MalformedStringError),
    MalformedDirectiveLocation(usize, String, usize),
    VariableInConstPosition(usize, String, usize),
}
