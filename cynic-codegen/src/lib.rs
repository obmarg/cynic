pub mod enum_derive;
pub mod enum_derive_2;
pub mod fragment_arguments_derive;
pub mod fragment_derive;
pub mod fragment_derive_2;
pub mod inline_fragments_derive;
pub mod inline_fragments_derive_2;
pub mod input_object_derive;
pub mod scalar_derive;
pub mod scalar_derive_2;
pub mod schema_for_derives;
pub mod use_schema;
pub mod use_schema2;

mod error;
mod field_argument;
mod field_type;
mod generic_param;
mod idents;
mod module;
mod schema;
mod suggestions;
mod type_validation;
mod type_validation_2;

pub use idents::RenameAll;

use error::Errors;
use field_argument::FieldArgument;
use field_type::FieldType;
use idents::Ident;
use schema::{load_schema, SchemaLoadError, TypeIndex};

pub fn output_schema_module(
    schema: impl AsRef<std::path::Path>,
    output_path: impl AsRef<std::path::Path>,
) -> Result<(), SchemaLoadError> {
    use std::io::Write;
    use use_schema::QueryDslParams;

    let tokens = use_schema::use_schema(QueryDslParams {
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
            .args(&["fmt", "--", filename.to_str().unwrap()])
            .spawn()
            .expect("failed to execute process");
    }
}
