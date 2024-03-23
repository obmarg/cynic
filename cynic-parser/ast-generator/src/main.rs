mod exts;
mod file;
mod idents;
mod object;
mod union;

use indexmap::IndexMap;
use indoc::formatdoc;
use itertools::Itertools;

use cynic_parser::type_system::{Definition, TypeDefinition};

use crate::{exts::FileDirectiveExt, file::imports};

fn main() -> anyhow::Result<()> {
    eprintln!("{:?}", std::env::current_dir());
    for module in ["executable"] {
        let document = std::fs::read_to_string(format!(
            "cynic-parser/ast-generator/domain/{module}.graphql"
        ))?;

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
            .values()
            .map(|model| {
                let output = match model {
                    TypeDefinition::Object(object) => object::object_output(*object, &model_index)?,
                    TypeDefinition::Scalar(_) => {
                        return Ok(None);
                    }
                    TypeDefinition::Union(union) => union::union_output(*union, &model_index)?,
                    _ => anyhow::bail!("unsupported definition"),
                };

                Ok(Some((model.file_name(), output)))
            })
            .filter_map(Result::transpose)
            .collect::<Result<Vec<(_, _)>, _>>()
            .unwrap()
            .into_iter()
            .into_group_map();

        for (file_name, output) in outputs {
            let requires = output
                .iter()
                .flat_map(|entity| entity.requires.clone().into_iter())
                .collect();
            let current_entities = output.iter().map(|entity| entity.id.clone()).collect();

            let id_trait = match module {
                "executable" => "ExecutableId",
                _ => todo!(),
            };
            let imports = imports(requires, current_entities, id_trait).unwrap();

            let doc = format_code(formatdoc!(
                r#"
                {imports}

                {}
                "#,
                output
                    .into_iter()
                    .map(|entity| entity.contents)
                    .join("\n\n")
            ))
            .unwrap();

            std::fs::write(
                format!("cynic-parser/ast-generator/output/{module}/{file_name}.rs"),
                doc,
            )
            .unwrap();
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
