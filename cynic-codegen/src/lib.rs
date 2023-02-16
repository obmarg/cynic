#![deny(rust_2018_idioms)]
pub mod enum_derive;
pub mod fragment_derive;
pub mod inline_fragments_derive;
pub mod input_object_derive;
pub mod query_variables_derive;
pub mod scalar_derive;
pub mod schema_for_derives;
pub mod use_schema;

mod error;
mod idents;
mod schema;
mod suggestions;
mod types;

pub use idents::RenameAll;

use error::Errors;
use schema::load_schema;

pub fn output_schema_module(
    schema: impl AsRef<std::path::Path>,
    output_path: impl AsRef<std::path::Path>,
) -> Result<(), Errors> {
    use std::io::Write;
    use use_schema::UseSchemaParams;

    let tokens = use_schema::use_schema(UseSchemaParams {
        schema_filename: schema.as_ref().to_str().unwrap().to_string(),
    })?;

    {
        let mut out = std::fs::File::create(output_path.as_ref()).unwrap();
        write!(&mut out, "{}", tokens).unwrap();
    }

    format_code(output_path.as_ref());

    Ok(())
}

#[allow(unused_variables)]
fn format_code(filename: &std::path::Path) {
    #[cfg(feature = "rustfmt")]
    {
        std::process::Command::new("cargo")
            .args(["fmt", "--", filename.to_str().unwrap()])
            .spawn()
            .expect("failed to execute process");
    }
}
