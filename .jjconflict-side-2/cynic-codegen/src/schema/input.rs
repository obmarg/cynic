use crate::schema::load_schema;

use super::parser::SchemaLoadError;

pub enum SchemaInput {
    Document(cynic_parser::TypeSystemDocument),
    #[cfg(feature = "rkyv")]
    Archive(Vec<u8>),
}

// Public API
impl SchemaInput {
    /// Parses a SchemaInput from a filename passed to a macro
    pub(crate) fn default() -> Result<SchemaInput, SchemaLoadError> {
        Self::from_schema_name("default").map_err(|error| match error {
            SchemaLoadError::NamedSchemaNotFound(_) => SchemaLoadError::DefaultSchemaNotFound,
            SchemaLoadError::UnknownOutDirWithNamedSchema(_) => {
                SchemaLoadError::UnknownOutDirWithDefaultSchema
            }
            _ => error,
        })
    }

    /// Parses a SchemaInput from a name passed to a macro
    pub(crate) fn from_schema_name(name: &str) -> Result<SchemaInput, SchemaLoadError> {
        let out_dir = std::env::var("OUT_DIR")
            .map_err(|_| SchemaLoadError::UnknownOutDirWithNamedSchema(name.to_string()))?;

        let mut path = std::path::PathBuf::from(out_dir);
        path.push("cynic-schemas");
        #[cfg(feature = "rkyv")]
        let extension = "rkyv";
        #[cfg(not(feature = "rkyv"))]
        let extension = "graphql";
        path.push(format!("{name}.{extension}"));
        if !path.exists() {
            return Err(SchemaLoadError::NamedSchemaNotFound(name.to_string()));
        }

        #[cfg(feature = "rkyv")]
        {
            Self::from_rkyv_bytes(std::fs::read(path)?)
        }
        #[cfg(not(feature = "rkyv"))]
        {
            let string = std::fs::read_to_string(path)?;
            Self::from_sdl(&string)
        }
    }

    /// Parses a SchemaInput from a filename passed to a macro
    pub(crate) fn from_schema_path(
        path: impl AsRef<std::path::Path>,
    ) -> Result<SchemaInput, SchemaLoadError> {
        let path = path.as_ref();
        if let Some(ast) = document_from_path(path)? {
            return Ok(SchemaInput::Document(ast));
        }
        return Err(SchemaLoadError::FileNotFound(
            path.to_string_lossy().to_string(),
        ));
    }

    pub fn from_sdl(sdl: &str) -> Result<SchemaInput, SchemaLoadError> {
        load_schema(sdl).map(SchemaInput::Document)
    }

    #[cfg(feature = "rkyv")]
    pub fn from_rkyv_bytes(bytes: Vec<u8>) -> Result<SchemaInput, SchemaLoadError> {
        use crate::schema::type_index::optimised::OptimisedTypes;
        // Typecheck here so we can be sure it's safe later on
        rkyv::check_archived_root::<OptimisedTypes<'_>>(&bytes)
            .map_err(|e| SchemaLoadError::ParseError(e.to_string()))?;

        Ok(SchemaInput::Archive(bytes))
    }
}

fn document_from_path(
    filename: impl AsRef<std::path::Path>,
) -> Result<Option<cynic_parser::TypeSystemDocument>, SchemaLoadError> {
    use std::path::PathBuf;
    let mut pathbuf = PathBuf::new();

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        pathbuf.push(manifest_dir);
    } else {
        pathbuf.push(std::env::current_dir()?);
    }
    pathbuf.push(filename);

    if pathbuf.exists() {
        load_schema(&std::fs::read_to_string(pathbuf)?).map(Some)
    } else {
        Ok(None)
    }
}
