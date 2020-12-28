use crate::schema::{EnumDetails, InputField};

mod argument_struct;
mod indent;
pub mod query_fragment;

pub use argument_struct::{ArgumentStruct, ArgumentStructField};
pub use indent::indented;
pub use query_fragment::QueryFragment;

pub struct Output<'query, 'schema> {
    pub query_fragments: Vec<QueryFragment<'query, 'schema>>,
    pub input_objects: Vec<InputObject<'schema>>,
    pub enums: Vec<EnumDetails<'schema>>,
    pub scalars: Vec<Scalar<'schema>>,
    pub argument_structs: Vec<ArgumentStruct<'query, 'schema>>,
}

pub struct Scalar<'schema>(pub &'schema str);

#[derive(Debug, PartialEq)]
pub struct InputObject<'schema> {
    pub name: String,
    pub fields: Vec<InputField<'schema>>,
}
