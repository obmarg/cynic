use crate::schema::load_schema;

use super::{parser::SchemaLoadError, Document};

pub enum SchemaInput {
    Document(Document),
    #[cfg(feature = "rkyv")]
    Archive(Vec<u8>),
}

// Public API
impl SchemaInput {
    /// Parses a SchemaInput from a filename passed to a macro
    pub(crate) fn from_schema_path(
        path: impl AsRef<std::path::Path>,
    ) -> Result<SchemaInput, SchemaLoadError> {
        let path = path.as_ref();
        if let Some(document) = document_from_path(path)? {
            return Ok(SchemaInput::Document(document));
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
) -> Result<Option<Document>, SchemaLoadError> {
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
