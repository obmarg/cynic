use std::fmt::{self, Display, Write};

use cynic::InputObject;

use crate::{
    EnumType, Field, FieldType, InputObjectType, InterfaceType, ObjectType, ScalarType, Type,
    UnionType, WrappingType,
};

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
                writeln!(f, "{}", DescriptionDisplay(description.as_deref()))?;
                writeln!(f, "scalar {name}")
            }
            Type::Object(ObjectType {
                name,
                description,
                fields,
                interfaces,
            }) => {
                writeln!(f, "{}", DescriptionDisplay(description.as_deref()))?;
                writeln!(f, "type {name} {{")?;
                for field in fields {
                    writeln!(f, "{}", FieldDisplay(field))?;
                }
                writeln!(f, "}}")
            }
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

struct DescriptionDisplay<'a>(Option<&'a str>);

impl Display for DescriptionDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = self.0 {
            writeln!(f, r#"""""#)?;
            writeln!(f, "{description}")?;
            writeln!(f, r#"""""#)?;
        }

        Ok(())
    }
}

struct FieldDisplay<'a>(&'a Field);

impl Display for FieldDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Field {
            name,
            description,
            args,
            ty,
            deprecated,
        } = self.0;
        writeln!(f, "{name}: {}", FieldTypeDisplay(ty))
    }
}

struct FieldTypeDisplay<'a>(&'a FieldType);

impl Display for FieldTypeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let FieldType { wrapping, name } = self.0;
        let wrapping_types = wrapping.into_iter().collect::<Vec<_>>();
        for wrapping_type in &wrapping_types {
            match wrapping_type {
                WrappingType::List => write!(f, "[")?,
                WrappingType::NonNull => {}
            }
        }
        writeln!(f, "{name}")?;
        for wrapping_type in wrapping_types.iter().rev() {
            match wrapping_type {
                WrappingType::List => write!(f, "]")?,
                WrappingType::NonNull => write!(f, "?")?,
            }
        }

        Ok(())
    }
}

pub fn indented<D>(f: &mut D) -> indenter::Indented<'_, D> {
    indenter::indented(f).with_str("  ")
}
