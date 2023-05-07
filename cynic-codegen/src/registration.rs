use std::{
    io::Write,
    path::{Path, PathBuf},
};

use crate::schema::{Schema, SchemaInput};

/// Registers a schema with cynic-codegen with the given name
///
/// This will prepare the schema for use and write intermediate files
/// into the current crates target directory.  You can then refer to
/// the schema by name when working with cynics macros.
///
/// This is designed to be called from `build.rs`
pub fn register_schema(name: &str) -> SchemaRegistrationBuilder<'_> {
    SchemaRegistrationBuilder { name }
}

#[derive(thiserror::Error, Debug)]
#[error("Could not register schema with cynic")]
pub enum SchemaRegistrationError {
    #[error("IOError: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Could not find the OUT_DIR environment variable, which should be set by cargo")]
    OutDirNotSet,
    #[error("Errors when parsing schema: {0}")]
    SchemaErrors(String),
}

#[must_use]
/// An incomplete schema registration.
///
/// Call one of the methods on this type to provide the schema details
pub struct SchemaRegistrationBuilder<'a> {
    name: &'a str,
}

impl<'a> SchemaRegistrationBuilder<'a> {
    /// Pulls schema information from the SDL file at `path`
    pub fn from_sdl_file(
        self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<SchemaRegistration<'a>, SchemaRegistrationError> {
        let SchemaRegistrationBuilder { name } = self;
        fn inner<'a>(
            name: &'a str,
            path: &Path,
        ) -> Result<SchemaRegistration<'a>, SchemaRegistrationError> {
            let data = std::fs::read(path)?;
            let registration = SchemaRegistration {
                name,
                data,
                format: Format::Sdl,
            };
            registration.write(&registration.filename()?)?;
            registration.write_schema_module()?;
            cargo_rerun_if_changed(path.as_os_str().to_str().expect("utf8 paths"));
            Ok(registration)
        }
        inner(name, path.as_ref())
    }
}

/// A complete schema registration.
///
/// Additional methods can be called on this to
pub struct SchemaRegistration<'a> {
    name: &'a str,
    format: Format,
    data: Vec<u8>,
}

// Public API
impl SchemaRegistration<'_> {
    /// Registers this schema as the default.
    ///
    /// The default schema (if any) will be used when you don't provide a schema
    /// name to any of the cynic macros.
    ///
    /// You should only call this once per crate - any subsequent calls will overwrite
    /// the default.
    pub fn as_default(self) -> Result<Self, SchemaRegistrationError> {
        self.write(&default_filename(self.format, &out_dir()?))?;
        Ok(self)
    }
}

// Private API
impl SchemaRegistration<'_> {
    fn write(&self, filename: &Path) -> Result<(), SchemaRegistrationError> {
        std::fs::create_dir_all(filename.parent().expect("filename to have a parent"))?;
        Ok(std::fs::write(filename, &self.data)?)
    }

    fn write_schema_module(&self) -> Result<(), SchemaRegistrationError> {
        use crate::use_schema::use_schema_impl;

        let document_string = String::from_utf8(self.data.clone()).expect("schema to be utf8");
        let document = crate::schema::load_schema(&document_string)
            .map_err(|error| SchemaRegistrationError::SchemaErrors(error.to_string()))?
            .into_static();

        let schema = Schema::new(SchemaInput::Document(document.clone()))
            .validate()
            .map_err(|errors| SchemaRegistrationError::SchemaErrors(errors.to_string()))?;

        let tokens = use_schema_impl(&document, schema)
            .map_err(|errors| SchemaRegistrationError::SchemaErrors(errors.to_string()))?;

        let schema_module_filename = schema_module_filename(self.name, &out_dir()?);
        std::fs::create_dir_all(
            schema_module_filename
                .parent()
                .expect("filename to have a parent"),
        )?;

        let mut out = std::fs::File::create(&schema_module_filename)?;
        write!(&mut out, "{}", tokens)?;

        Ok(())
    }

    fn filename(&self) -> Result<PathBuf, SchemaRegistrationError> {
        let out_dir = out_dir()?;
        Ok(registration_filename(self.name, self.format, &out_dir))
    }
}

fn cargo_rerun_if_changed(name: &str) {
    println!("cargo:rerun-if-changed={name}");
}

pub(super) fn out_dir() -> Result<String, SchemaRegistrationError> {
    let out_dir = std::env::var("OUT_DIR").map_err(|_| SchemaRegistrationError::OutDirNotSet)?;
    Ok(out_dir)
}

pub(super) fn schema_module_filename(name: &str, out_dir: &str) -> PathBuf {
    let mut path = PathBuf::from(out_dir);
    path.push("cynic-schemas");
    path.push(format!("{name}.rs",));

    path
}

fn registration_filename(name: &str, format: Format, out_dir: &str) -> PathBuf {
    let extension = match format {
        Format::Sdl => "graphql",
    };
    let mut path = PathBuf::from(out_dir);
    path.push("cynic-schemas");
    path.push(format!("{name}.{extension}",));

    path
}

fn default_filename(format: Format, out_dir: &str) -> PathBuf {
    let extension = match format {
        Format::Sdl => "graphql",
    };
    let mut path = PathBuf::from(out_dir);
    path.push("cynic-schemas");
    path.push(format!("default.{extension}",));

    path
}

#[derive(Clone, Copy)]
enum Format {
    Sdl,
}
