pub mod ast;
mod lexer;
mod parser;
mod syntax;

pub use syntax::SyntaxNode;

use parser::parse;

pub fn parse_query_document(text: &str) -> Option<ast::Document> {
    use ast::AstNode;

    // TODO: This is not very nice, tidy it up
    let parse = parse(text);
    if !parse.errors.is_empty() {
        panic!("Errors: {:?}", parse.errors)
    }

    ast::Document::cast(parse.syntax())
}
