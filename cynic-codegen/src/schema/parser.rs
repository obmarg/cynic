use crate::TypeIndex;

// TODO: It would be good to make these use &'str since this is compile time
// and max speed is best.

// Alias all the graphql_parser schema types so we don't have to specify generic parameters
// everywhere
pub type Document = graphql_parser::schema::Document<'static, String>;
pub type Type = graphql_parser::schema::Type<'static, String>;
pub type Field = graphql_parser::schema::Field<'static, String>;
pub type Definition = graphql_parser::schema::Definition<'static, String>;
pub type TypeDefinition = graphql_parser::schema::TypeDefinition<'static, String>;
pub type ObjectType = graphql_parser::schema::ObjectType<'static, String>;
pub type EnumType = graphql_parser::schema::EnumType<'static, String>;
pub type InterfaceType = graphql_parser::schema::InterfaceType<'static, String>;
pub type InputObjectType = graphql_parser::schema::InputObjectType<'static, String>;
pub type ScalarType = graphql_parser::schema::ScalarType<'static, String>;
pub type UnionType = graphql_parser::schema::UnionType<'static, String>;
pub type InputValue = graphql_parser::schema::InputValue<'static, String>;
pub type EnumValue = graphql_parser::schema::EnumValue<'static, String>;

/// Loads a schema from a filename, relative to CARGO_MANIFEST_DIR if it's set.
pub fn load_schema(filename: impl AsRef<std::path::Path>) -> Result<Document, SchemaLoadError> {
    use std::path::PathBuf;
    let mut pathbuf = PathBuf::new();

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        pathbuf.push(manifest_dir);
    } else {
        pathbuf.push(std::env::current_dir()?);
    }
    pathbuf.push(filename);

    let schema = std::fs::read_to_string(&pathbuf)
        .map_err(|_| SchemaLoadError::FileNotFound(pathbuf.to_str().unwrap().to_string()))?;

    parse_schema(&schema)
}

pub(crate) fn parse_schema(schema: &str) -> Result<Document, SchemaLoadError> {
    let borrowed_schema = graphql_parser::schema::parse_schema::<String>(schema)?;
    Ok(schema_into_static(borrowed_schema))
}

fn schema_into_static<'a>(
    doc: graphql_parser::schema::Document<'a, String>,
) -> graphql_parser::schema::Document<'static, String> {
    // A workaround until https://github.com/graphql-rust/graphql-parser/pull/33 is
    // merged upstream.

    // A document that uses Strings for it's data should be safe to cast to 'static lifetime
    // as there's nothing inside it to reference 'a anyway
    unsafe { std::mem::transmute::<_, Document>(doc) }
}

#[derive(Debug, PartialEq, Clone)]
pub enum SchemaLoadError {
    IoError(String),
    ParseError(String),
    FileNotFound(String),
}

impl SchemaLoadError {
    pub fn into_syn_error(self, schema_span: proc_macro2::Span) -> syn::Error {
        let message = match self {
            SchemaLoadError::IoError(e) => format!("Could not load schema file: {}", e),
            SchemaLoadError::ParseError(e) => format!("Could not parse schema file: {}", e),
            SchemaLoadError::FileNotFound(e) => format!("Could not find file: {}", e),
        };

        syn::Error::new(schema_span, message)
    }
}

impl From<graphql_parser::schema::ParseError> for SchemaLoadError {
    fn from(e: graphql_parser::schema::ParseError) -> SchemaLoadError {
        SchemaLoadError::ParseError(e.to_string())
    }
}

impl From<std::io::Error> for SchemaLoadError {
    fn from(e: std::io::Error) -> SchemaLoadError {
        SchemaLoadError::IoError(e.to_string())
    }
}

/// Extension trait for the schema Type type
pub trait TypeExt {
    fn inner_name(&self) -> &str;
}

impl TypeExt for Type {
    fn inner_name(&self) -> &str {
        match self {
            Type::NamedType(s) => s,
            Type::ListType(inner) => inner.inner_name(),
            Type::NonNullType(inner) => inner.inner_name(),
        }
    }
}

pub trait ScalarTypeExt {
    fn is_builtin(&self) -> bool;
}

impl ScalarTypeExt for ScalarType {
    fn is_builtin(&self) -> bool {
        matches!(self.name.as_ref(), "String" | "Int" | "Boolean" | "ID")
    }
}
