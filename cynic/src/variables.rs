use std::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub enum VariableType {
    List(&'static VariableType),
    Nullable(&'static VariableType),
    Named(&'static str),
}

pub trait QueryVariables {
    type Fields;

    const VARIABLES: &'static [(&'static str, VariableType)];
}

// TODO: Figure out if this makes sense.
impl QueryVariables for () {
    type Fields = ();

    const VARIABLES: &'static [(&'static str, VariableType)] = &[];
}

// TODO: Think about this name & where we should put it
pub struct VariableDefinition<Variables, Type> {
    pub name: &'static str,
    phantom: PhantomData<fn() -> (Variables, Type)>,
}

impl<Variables, Type> VariableDefinition<Variables, Type> {
    pub fn new(name: &'static str) -> Self {
        VariableDefinition {
            name,
            phantom: PhantomData,
        }
    }
}
