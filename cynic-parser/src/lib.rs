use lalrpop_util::lalrpop_mod;

mod ast;
mod lexer;
mod printer;

pub use lexer::Lexer;
pub use schema::ObjectDefinitionParser;

// TODO: Make this more senseible
pub use ast::Ast;

lalrpop_mod!(pub schema);

pub fn parse_type_system_document(input: &str) -> Ast {
    let lexer = lexer::Lexer::new(input);
    let mut ast = Ast::new();

    schema::TypeSystemDocumentParser::new()
        .parse(input, &mut ast, lexer)
        .expect("TODO: error handling");

    ast
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        insta::assert_snapshot!(
            parse_type_system_document("schema { query:Query }").to_sdl(),
            @r###"
        schema {
          query: Query
        }
        "###
        );
    }

    #[test]
    fn test_basic_object() {
        insta::assert_snapshot!(
            parse_type_system_document(r#"
                type MyType implements Blah & Bloo @hello {
                    field: Whatever @hello(name: ["string"]),
                    other: [[Int!]]!
                }"#
            ).to_sdl(),
            @r###"
        type MyType implements Blah & Bloo @hello {
          field: Whatever@hello(name: ("string"))
          other: [[Int!]]!
        }
        "###
        );
    }

    #[test]
    fn test_schema_field() {
        // Use a keyword as a field name and make sure it's fine
        insta::assert_snapshot!(
            parse_type_system_document( "type MyType { query: String }").to_sdl(),
            @r###"
        type MyType  {
          query: String
        }
        "###
        )
    }

    #[test]
    fn test_input() {
        insta::assert_snapshot!(
            parse_type_system_document(
                r#"
                "I am a description"
                input MyType @hello { query: String = "Hello" }
                "#
            ).to_sdl(),
            @r###"
        input MyType @hello {
          query: String = "Hello"
        }
        "###
        );
    }
}
