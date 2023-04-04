use std::fmt::{self, Display, Write};

use cynic::InputObject;

use crate::{EnumType, InputObjectType, InterfaceType, ObjectType, ScalarType, Type, UnionType};

use super::Schema;

impl Schema {
    pub fn write_sdl(&self, f: &dyn Write) -> std::io::Result<()> {
        todo!()
    }

    pub fn to_sdl(&self) -> String {
        let mut output = String::new();
        self.write_sdl(&output)
            .expect("Writing to a string shouldn't fail");
        output
    }
}

struct SchemaDisplay<'a>(&'a Schema);

impl Display for SchemaDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let schema = &self.0;
        writeln!(f, "schema {{")?;
        {
            let f = &mut indented(f);
            writeln!(f, "query: {}", schema.query_type)?;
            if let Some(mutation_type) = &schema.mutation_type {
                writeln!(f, "mutation: {}", mutation_type)?;
            }

            if let Some(subscription_type) = &schema.subscription_type {
                writeln!(f, "subscription: {}", subscription_type)?;
            }
        }
        writeln!(f, "}}")?;

        for ty in &schema.types {
            let ty = TypeDisplay(ty);
            writeln!(f, "{ty}")?;
        }

        Ok(())
    }
}

struct TypeDisplay<'a>(&'a Type);

impl std::fmt::Display for TypeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ty = self.0;

        match ty {
            Type::Scalar(ScalarType { name, description }) => {
                writeln!(f, "{}", DescriptionOutput(description.as_deref()))?;
                writeln!(f, "scalar {name}")
            }
            Type::Object(ObjectType {
                name,
                description,
                fields,
                interfaces,
            }) => todo!(),
            Type::InputObject(InputObjectType {
                name,
                description,
                fields,
            }) => todo!(),
            Type::Enum(EnumType {
                name,
                description,
                values,
            }) => todo!(),
            Type::Interface(InterfaceType {
                name,
                description,
                fields,
                interfaces,
                possible_types,
            }) => todo!(),
            Type::Union(UnionType {
                name,
                description,
                possible_types,
            }) => todo!(),
        }
    }
}

struct DescriptionOutput<'a>(Option<&'a str>);

impl Display for DescriptionOutput<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = self.0 {
            writeln!(f, r#"""""#)?;
            writeln!(f, "{description}")?;
            writeln!(f, r#"""""#)?;
        }

        Ok(())
    }
}

pub fn indented<D>(f: &mut D) -> indenter::Indented<'_, D> {
    indenter::indented(f).with_str("  ")
}
