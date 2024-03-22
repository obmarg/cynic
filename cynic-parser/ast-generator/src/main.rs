mod idents;
mod object;

use indexmap::IndexMap;
use itertools::Itertools;
use proc_macro2::{Ident, Span};
use quote::{quote, TokenStreamExt};
use std::{collections::HashMap, ops::Deref};

use cynic_parser::type_system::{
    readers::{FieldDefinition, ObjectDefinition},
    Definition, TypeDefinition,
};

use crate::idents::IdIdent;

fn main() -> anyhow::Result<()> {
    eprintln!("{:?}", std::env::current_dir());
    for module in ["executable", "type_system"] {
        let document =
            std::fs::read_to_string("cynic-parser/ast-generator/domain/executable.graphql")?;

        let domain = match cynic_parser::parse_type_system_document(&document) {
            Ok(domain) => domain,
            Err(error) => {
                eprintln!("Error parsing document");
                eprintln!("{}", error.to_report(&document));
                return Err(anyhow::anyhow!(""));
            }
        };

        let mut model_index = IndexMap::new();

        for definition in domain.definitions() {
            match definition {
                Definition::Type(ty) => {
                    model_index.insert(ty.name(), ty);
                }
                _ => anyhow::bail!("unsupported definition"),
            }
        }

        let outputs = model_index
            .iter()
            .map(|(name, model)| {
                let output = match model {
                    TypeDefinition::Object(object) => object::object_output(*object, &model_index)?,
                    TypeDefinition::Scalar(_) => {
                        return Ok(None);
                    }
                    TypeDefinition::Union(_) => {
                        // TODO
                        return Ok(None);
                    }
                    _ => anyhow::bail!("unsupported definition"),
                };

                let file_name = model
                    .directives()
                    .find(|directive| directive.name() == "file")
                    .and_then(|directive| directive.arguments().next()?.value().as_str())
                    .unwrap_or(name);

                Ok(Some((file_name, output)))
            })
            .filter_map(Result::transpose)
            .collect::<Result<Vec<(_, _)>, _>>()
            .unwrap()
            .into_iter()
            .into_group_map();

        for (name, output) in outputs {
            println!("Output for {name}:\n\n{output}\n\n");
        }
    }

    Ok(())
}

fn format_code(text: impl ToString) -> anyhow::Result<String> {
    use xshell::{cmd, Shell};
    let sh = Shell::new()?;

    let stdout = cmd!(sh, "rustfmt").stdin(&text.to_string()).read()?;

    Ok(stdout)
}
