pub mod ids;

mod operation;

struct Ast {
    operations: Vec<operation::OperationDefinitionRecord>,
}
