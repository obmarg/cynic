use graphql_parser::{query, schema};

mod query_parsing;
mod type_ext;
mod type_index;

use query_parsing::PotentialStruct;
use type_ext::TypeExt;
use type_index::TypeIndex;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Query document not supported: {0}")]
    UnsupportedQueryDocument(String),

    #[error("could not parse query document: {0}")]
    QueryParseError(#[from] query::ParseError),

    #[error("could not parse schema document: {0}")]
    SchemaParseError(#[from] schema::ParseError),

    #[error("could not find field `{0}` on `{1}`")]
    UnknownField(String, String),
}

pub fn document_to_fragment_structs(
    query: impl AsRef<str>,
    schema: impl AsRef<str>,
) -> Result<String, Error> {
    let schema = graphql_parser::parse_schema::<&str>(schema.as_ref())?;
    let query = graphql_parser::parse_query::<&str>(query.as_ref())?;

    let possible_structs = query_parsing::parse_query_document(&query)?;
    let type_index = TypeIndex::from_schema(&schema);

    for st in possible_structs {
        match st {
            PotentialStruct::QueryFragment(fragment) => {
                let name = if fragment.path.is_empty() {
                    // We have a root query type here, so lets make up a name
                    "Root"
                } else {
                    type_index.type_for_path(&fragment.path)?.inner_name()
                };

                println!("struct {} {{", name);

                for field in fragment.fields {
                    let mut field_path = fragment.path.clone();
                    field_path.push(field.name);
                    let field_ty = type_index.type_for_path(&field_path)?;

                    println!("\t{}: {},", field.name, field_ty.type_spec())
                }
                println!("}}\n");
            }
            _ => {}
        }
    }

    Ok("".to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
