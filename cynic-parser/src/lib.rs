use lalrpop_util::lalrpop_mod;

mod ast;
mod lexer;
mod printer;

pub use lexer::Lexer;
pub use schema::ObjectDefinitionParser;

// TODO: Make this more senseible
pub use ast::Ast;

lalrpop_mod!(pub schema);

// TODO: Wonder if the parser should intern strings?
// Maybe?

#[cfg(test)]
mod tests {
    use crate::ast::Ast;

    use super::*;

    #[test]
    fn it_works() {
        let input = "schema { query:Query }";
        let lexer = lexer::Lexer::new(input);
        let mut builder = Ast::new();
        insta::assert_debug_snapshot!(schema::SchemaDefinitionParser::new().parse(input, &mut builder, lexer).unwrap(), @r###"
        Schema {
            query: "Query",
        }
        "###)
    }

    #[test]
    fn test_basic_object() {
        let input = "type MyType { field: Whatever, field: Whatever }";
        let lexer = lexer::Lexer::new(input);
        let mut builder = Ast::new();
        insta::assert_debug_snapshot!(schema::ObjectDefinitionParser::new().parse(input, &mut builder, lexer).unwrap(), @r###"
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

    #[test]
    fn test_schema_field() {
        // Use a keyword as a field name and make sure it's fine
        let input = "type MyType { query: String }";
        let lexer = lexer::Lexer::new(input);
        let mut builder = Ast::new();
        insta::assert_debug_snapshot!(schema::ObjectDefinitionParser::new().parse(input, &mut builder, lexer).unwrap(), @r###"
        Object {
            name: "MyType",
            fields: [
                Field {
                    name: "query",
                    ty: "String",
                },
            ],
        }
        "###)
    }

    #[test]
    fn test_input() {
        // Use a keyword as a field name and make sure it's fine
        let input = r#"
        "I am a description"
        type MyType { query: String }
        "#;
        let lexer = lexer::Lexer::new(input);
        let mut builder = Ast::new();
        insta::assert_debug_snapshot!(schema::TypeSystemDocumentParser::new().parse(input, &mut builder, lexer).unwrap(), @r###"
        Object {
            name: "MyType",
            fields: [
                Field {
                    name: "query",
                    ty: "String",
                },
            ],
        }
        "###)
    }
}
