use lalrpop_util::lalrpop_mod;

mod ast;
mod lexer;

lalrpop_mod!(pub schema);

// TODO: Wonder if the parser should intern strings?
// Maybe?

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let input = "schema { query:Query }";
        let lexer = lexer::Lexer::new(input);
        insta::assert_debug_snapshot!(schema::SchemaParser::new().parse(input, lexer).unwrap())
    }

    #[test]
    fn test_basic_object() {
        let input = "type MyType { field: Whatever, field: Whatever }";
        let lexer = lexer::Lexer::new(input);
        insta::assert_debug_snapshot!(schema::ObjectParser::new().parse(input, lexer).unwrap(), @r###"
        Object {
            name: "MyType",
            fields: [
                Field {
                    name: "field",
                    ty: "Whatever",
                },
                Field {
                    name: "field",
                    ty: "Whatever",
                },
            ],
        }
        "###)
    }
}
