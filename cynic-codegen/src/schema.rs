use crate::{Error, FieldArgument, TypeIndex};

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
pub type UnionType = graphql_parser::schema::UnionType<'static, String>;
pub type InputValue = graphql_parser::schema::InputValue<'static, String>;
pub type EnumValue = graphql_parser::schema::EnumValue<'static, String>;

/// Loads a schema from a filename, relative to CARGO_MANIFEST_DIR if it's set.
pub fn load_schema(filename: impl AsRef<std::path::Path>) -> Result<Document, Error> {
    use std::path::PathBuf;
    let mut pathbuf = PathBuf::new();

    let filename = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        pathbuf.push(manifest_dir);
        pathbuf.push(filename);
        pathbuf.as_path()
    } else {
        filename.as_ref()
    };

    let schema = std::fs::read_to_string(filename)?;
    let borrowed_schema = graphql_parser::schema::parse_schema::<String>(&schema)?;
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

/// Extension trait for the schema Field type
pub trait FieldExt {
    fn required_arguments(&self) -> Vec<InputValue>;
    fn optional_arguments(&self) -> Vec<InputValue>;
}

impl FieldExt for Field {
    fn required_arguments(&self) -> Vec<InputValue> {
        self.arguments
            .iter()
            .filter(|arg| {
                // Note: We're passing an empty TypeIndex in here, but that's OK as
                // we only want to know if things are required
                FieldArgument::from_input_value(arg, &TypeIndex::empty()).is_required()
            })
            .map(|a| a.clone())
            .collect()
    }

    fn optional_arguments(&self) -> Vec<InputValue> {
        self.arguments
            .iter()
            .filter(|arg| {
                // Note: We're passing an empty TypeIndex in here, but that's OK as
                // we only want to know if things are required
                !FieldArgument::from_input_value(arg, &TypeIndex::empty()).is_required()
            })
            .map(|a| a.clone())
            .collect()
    }
}

/// Extension trait for the schema Type type
pub trait TypeExt {
    fn to_graphql_string(&self) -> String;
}

impl TypeExt for Type {
    fn to_graphql_string(&self) -> String {
        match self {
            Type::NamedType(name) => name.clone(),
            Type::ListType(inner) => format!("[{}]", inner.to_graphql_string()),
            Type::NonNullType(inner) => format!("{}!", inner.to_graphql_string()),
        }
    }
}
