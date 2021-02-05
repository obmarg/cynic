use strsim::hamming;

use error::Errors;
use field_argument::FieldArgument;
use field_type::FieldType;
use ident::Ident;
pub use ident::RenameAll;
use schema::{load_schema, SchemaLoadError};
use type_index::TypeIndex;
use type_path::TypePath;

pub mod enum_derive;
pub mod fragment_arguments_derive;
pub mod fragment_derive;
pub mod inline_fragments_derive;
pub mod input_object_derive;
pub mod query_dsl;
pub mod query_module;
pub mod scalar_derive;

mod error;
mod field_argument;
mod field_type;
mod generic_param;
mod ident;
mod module;
mod schema;
mod type_index;
mod type_path;
mod type_validation;

pub fn output_query_dsl(
    schema: impl AsRef<std::path::Path>,
    output_path: impl AsRef<std::path::Path>,
) -> Result<(), SchemaLoadError> {
    use query_dsl::QueryDslParams;
    use std::io::Write;

    let tokens = query_dsl::query_dsl_from_schema(QueryDslParams {
        schema_filename: schema.as_ref().to_str().unwrap().to_string(),
    })?;

    {
        let mut out = std::fs::File::create(output_path.as_ref()).unwrap();
        write!(&mut out, "{}", tokens).unwrap();
    }

    format_code(output_path.as_ref());

    Ok(())
}

/// Using Hamming algorithm to guess possible similar fields.
pub fn guess_field(candidates: &Vec<String>, field_name: &str, k: usize) -> Option<String> {
    return candidates.iter().find(|x|
        match hamming(x.as_str(), field_name) {
            //For example, consider the code consisting of two codewords "000" and "111".
            //The hamming distance between these two words is 3, and therefore it is k=2 error detecting.
            //Which means that if one bit is flipped or two bits are flipped, the error can be detected.
            //If three bits are flipped, then "000" becomes "111" and the error can not be detected.
            Ok(distance) =>
                if distance <= k {
                    true
                } else {
                    false
                }
            Err(_) => false
        }).map(|x| x.to_owned());
}

pub fn format_guess(guess_field: Option<String>) -> String {
    return match guess_field {
        Some(v) => format!("According to the guess, what you need is {} ?", v),
        None => "".to_owned()
    };
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
