use lalrpop_util::lalrpop_mod;

mod ast;
mod lexer;

lalrpop_mod!(pub schema);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        insta::assert_debug_snapshot!(schema::SchemaParser::new()
            .parse("schema { query: Query }")
            .unwrap())
    }
}
