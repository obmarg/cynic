/// Loads a schema from a string
pub fn load_schema(sdl: &str) -> Result<cynic_parser::Ast, SchemaLoadError> {
    let ast = cynic_parser::parse_type_system_document(sdl);
    Ok(ast)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SchemaLoadError {
    IoError(String),
    ParseError(String),
    FileNotFound(String),
    NamedSchemaNotFound(String),
    DefaultSchemaNotFound,
    UnknownOutDirWithNamedSchema(String),
    UnknownOutDirWithDefaultSchema,
}

impl SchemaLoadError {
    pub fn into_syn_error(self, schema_span: proc_macro2::Span) -> syn::Error {
        syn::Error::new(schema_span, self.to_string())
    }
}

impl std::fmt::Display for SchemaLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaLoadError::IoError(e) => write!(f, "Could not load schema file: {}", e),
            SchemaLoadError::ParseError(e) => write!(f, "Could not parse schema file: {}", e),
            SchemaLoadError::FileNotFound(e) => write!(f, "Could not find file: {}", e),
            SchemaLoadError::NamedSchemaNotFound(_) => write!(
                f,
                "Could not find a schema with this name.  Have you registered it in build.rs?  {SCHEMA_DOCUMENTATION_TEXT}",
            ),
            SchemaLoadError::DefaultSchemaNotFound => {
                write!(f, "This derive is trying to use the default schema but it doesn't look like you've registered a default.  Please provide the `schema` argument or set a default in your build.rs.  {SCHEMA_DOCUMENTATION_TEXT}")
            }
            SchemaLoadError::UnknownOutDirWithNamedSchema(name) => {
                write!(f, "You requested a schema named {name} but it doesn't look like you've registered any schemas.  {SCHEMA_DOCUMENTATION_TEXT}")
            }
            SchemaLoadError::UnknownOutDirWithDefaultSchema => {
                write!(
                    f,
                    "This derive is trying to use the default schema, but it doesn't look like you've registered any schemas.  {SCHEMA_DOCUMENTATION_TEXT}"
                )
            }
        }
    }
}

// TODO: I could put a link in this
const SCHEMA_DOCUMENTATION_TEXT: &str =
    "See the cynic documentation on regsistering schemas if you need help.";

impl From<std::io::Error> for SchemaLoadError {
    fn from(e: std::io::Error) -> SchemaLoadError {
        SchemaLoadError::IoError(e.to_string())
    }
}
