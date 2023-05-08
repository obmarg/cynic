#![deny(rust_2018_idioms)]
pub mod enum_derive;
pub mod fragment_derive;
pub mod generics_for_serde;
pub mod inline_fragments_derive;
pub mod input_object_derive;
pub mod query_variables_derive;
pub mod registration;
pub mod scalar_derive;
pub mod schema_for_derives;
pub mod use_schema;

mod error;
mod idents;
mod schema;
mod suggestions;
mod types;

pub use self::{idents::RenameAll, registration::register_schema};

use {error::Errors, schema::load_schema};

#[deprecated(
    since = "3.0.0",
    note = "output_schema_module is deprecated.  See `register_schema` instead"
)]
pub fn output_schema_module(
    schema: impl AsRef<std::path::Path>,
    output_path: impl AsRef<std::path::Path>,
) -> Result<(), Errors> {
    use {std::io::Write, use_schema::UseSchemaParams};

    let tokens = use_schema::use_schema(UseSchemaParams {
        schema_filename: schema.as_ref().to_str().unwrap().to_string(),
    })?;

    {
        let mut out = std::fs::File::create(output_path.as_ref()).unwrap();
        write!(&mut out, "{}", tokens).unwrap();
    }

    format_code(output_path.as_ref()).ok();

    Ok(())
}

#[allow(unused_variables)]
fn format_code(filename: &std::path::Path) -> std::io::Result<()> {
    #[cfg(feature = "rustfmt")]
    {
        std::process::Command::new("cargo")
            .args(["fmt", "--", filename.to_str().unwrap()])
            .spawn()?;
    }
    Ok(())
}

fn variables_fields_path(variables: Option<&syn::Path>) -> Option<syn::Path> {
    variables.cloned().map(|mut variables| {
        // Find VariablesField struct name from Variables struct name
        if let Some(last) = variables.segments.last_mut() {
            last.ident = variables_fields_ident(&last.ident);
        }
        variables
    })
}

fn variables_fields_ident(ident: &syn::Ident) -> syn::Ident {
    proc_macro2::Ident::new(&format!("{}Fields", ident), ident.span())
}
