use lalrpop_util::lalrpop_mod;

mod ast;
mod lexer;

lalrpop_mod!(pub schema);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let input = "schema { query:Query }";
        let lexer = lexer::Lexer::new(input);
        insta::assert_debug_snapshot!(schema::SchemaParser::new().parse(input, lexer).unwrap())
    }
}
