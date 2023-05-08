use crate::schema::load_schema;

use super::{parser::SchemaLoadError, type_index::optimised::OptimisedTypes, Document};

pub enum SchemaInput {
    Document(Document),
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

    pub fn from_rkyv_bytes(bytes: Vec<u8>) -> Result<SchemaInput, SchemaLoadError> {
        // Typecheck here so we can be sure it's safe later on
        rkyv::check_archived_root::<OptimisedTypes<'_>>(&bytes)
            .map_err(|e| SchemaLoadError::ParseError(e.to_string()))?;

        Ok(SchemaInput::Archive(bytes))
    }
}

// Private API
// fn rkyv_from_outdir(
//     filename: &std::path::Path,
//     outdir: String,
// ) -> Result<Option<Vec<u8>>, SchemaLoadError> {
//     if filename.components().count() != 1 {
//         // We take a schema name for arkyvs, not a path
//         return Ok(None);
//     }
//     let mut path = std::path::PathBuf::from(outdir);
//     path.push("cynic");
//     path.push(format!("{}.rkyv", filename.to_string_lossy()));
//     if !path.exists() {
//         return Ok(None);
//     }
//     let bytes = std::fs::read(path)?;

//     Ok(Some(bytes))
// }

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

/// Loads a schema from a filename, relative to CARGO_MANIFEST_DIR if it's set.
#[cfg(nope)]
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

    Ok(add_typenames(parse_schema(&schema)?))
}
