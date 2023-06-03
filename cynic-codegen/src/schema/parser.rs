// TODO: It would be good to make these use &'str since this is compile time
// and max speed is best.

// TODO: It would be good to make these use &'str since this is compile time
// and max speed is best.

// Alias all the graphql_parser schema types so we don't have to specify generic parameters
// everywhere
pub type Document = graphql_parser::schema::Document<'static, String>;
pub type Type = graphql_parser::schema::Type<'static, String>;
pub type Field = graphql_parser::schema::Field<'static, String>;
pub type Definition = graphql_parser::schema::Definition<'static, String>;
pub type TypeDefinition = graphql_parser::schema::TypeDefinition<'static, String>;
pub type ScalarType = graphql_parser::schema::ScalarType<'static, String>;
pub type InputValue = graphql_parser::schema::InputValue<'static, String>;

/// Loads a schema from a string
pub fn load_schema(sdl: &str) -> Result<Document, SchemaLoadError> {
    Ok(add_typenames(parse_schema(sdl)?))
}

fn add_typenames(mut schema: Document) -> Document {
    for definition in &mut schema.definitions {
        match definition {
            Definition::TypeDefinition(TypeDefinition::Object(def)) => {
                def.fields.push(typename_field());
            }
            Definition::TypeDefinition(TypeDefinition::Interface(def)) => {
                def.fields.push(typename_field());
            }
            _ => {}
        }
    }
    schema
}

pub(crate) fn parse_schema(schema: &str) -> Result<Document, SchemaLoadError> {
    Ok(graphql_parser::schema::parse_schema::<String>(schema)?.into_static())
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

fn typename_field() -> Field {
    Field {
        position: graphql_parser::Pos { line: 0, column: 0 },
        description: None,
        name: "__typename".into(),
        arguments: vec![],
        field_type: Type::NonNullType(Box::new(Type::NamedType("String".into()))),
        directives: vec![],
    }
}
