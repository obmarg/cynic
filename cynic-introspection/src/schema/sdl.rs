use std::fmt::{self, Display, Write};

use crate::{
    Deprecated, EnumType, EnumValue, Field, FieldType, FieldWrapping, InputObjectType, InputValue,
    InterfaceType, ObjectType, ScalarType, Type, UnionType, WrappingType,
};

use super::Schema;

impl Schema {
    /// Writes the SDL for this schema into `f`
    pub fn write_sdl(&self, f: &mut dyn Write) -> std::fmt::Result {
        write!(f, "{}", SchemaDisplay(self))
    }

    /// Returns the SDL for this schema as a string
    pub fn to_sdl(&self) -> String {
        let mut output = String::new();
        self.write_sdl(&mut output)
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
            write!(f, "{ty}")?;
        }

        Ok(())
    }
}

struct TypeDisplay<'a>(&'a Type);

impl std::fmt::Display for TypeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ty = self.0;

        if ty.name().starts_with("__") {
            // Don't render introspection types.
            return Ok(());
        }

        match ty {
            Type::Scalar(
                scalar @ ScalarType {
                    name,
                    description,
                    specified_by_url,
                },
            ) => {
                if scalar.is_builtin() {
                    // Don't render built in scalars
                    return Ok(());
                }
                write!(f, "{}", DescriptionDisplay(description.as_deref()))?;
                writeln!(
                    f,
                    "scalar {name}{}\n",
                    SpecifiedByDisplay(specified_by_url.as_deref())
                )
            }
            Type::Object(ObjectType {
                name,
                description,
                fields,
                interfaces,
            }) => {
                write!(f, "{}", DescriptionDisplay(description.as_deref()))?;
                writeln!(f, "type {name}{} {{", ImplementsDisplay(interfaces))?;
                for field in fields {
                    writeln!(indented(f), "{}", FieldDisplay(field))?;
                }
                writeln!(f, "}}\n")
            }
            Type::InputObject(InputObjectType {
                name,
                description,
                fields,
            }) => {
                write!(f, "{}", DescriptionDisplay(description.as_deref()))?;
                writeln!(f, "input {name} {{")?;
                for field in fields {
                    writeln!(indented(f), "{}", InputValueDisplay(field))?;
                }
                writeln!(f, "}}\n")
            }
            Type::Enum(EnumType {
                name,
                description,
                values,
            }) => {
                write!(f, "{}", DescriptionDisplay(description.as_deref()))?;
                writeln!(f, "enum {name} {{")?;
                for value in values {
                    writeln!(indented(f), "{}", EnumValueDisplay(value))?;
                }
                writeln!(f, "}}\n")
            }
            Type::Interface(InterfaceType {
                name,
                description,
                fields,
                interfaces,
                possible_types: _,
            }) => {
                write!(f, "{}", DescriptionDisplay(description.as_deref()))?;
                writeln!(f, "interface {name}{} {{", ImplementsDisplay(interfaces))?;
                for field in fields {
                    writeln!(indented(f), "{}", FieldDisplay(field))?;
                }
                writeln!(f, "}}\n")
            }
            Type::Union(UnionType {
                name,
                description,
                possible_types,
            }) => {
                write!(f, "{}", DescriptionDisplay(description.as_deref()))?;
                write!(f, "union {name} = ")?;
                for (i, ty) in possible_types.iter().enumerate() {
                    if i != 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{ty}")?;
                }
                writeln!(f, "\n")
            }
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
        write!(f, "{}", DescriptionDisplay(description.as_deref()))?;
        write!(f, "{name}")?;
        if !args.is_empty() {
            writeln!(f, "(")?;
            for arg in args {
                writeln!(indented(f), "{}", InputValueDisplay(arg))?;
            }
            write!(f, ")")?;
        }
        write!(
            f,
            ": {} {}",
            FieldTypeDisplay(ty),
            DeprecatedDisplay(deprecated)
        )?;

        Ok(())
    }
}

struct ImplementsDisplay<'a>(&'a [String]);

impl Display for ImplementsDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let interfaces = &self.0;
        if interfaces.is_empty() {
            return Ok(());
        }
        write!(f, " implements ")?;
        for (i, interface) in interfaces.iter().enumerate() {
            if i != 0 {
                write!(f, " & ")?;
            }
            write!(f, "{interface}")?;
        }
        Ok(())
    }
}

struct FieldTypeDisplay<'a>(&'a FieldType);

impl Display for FieldTypeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let FieldType { wrapping, name } = self.0;
        write!(f, "{}", WrappingDisplay(wrapping, name))
    }
}

struct InputValueDisplay<'a>(&'a InputValue);

impl Display for InputValueDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let InputValue {
            name,
            description,
            ty,
            default_value,
        } = self.0;
        write!(f, "{}", DescriptionDisplay(description.as_deref()))?;
        write!(f, "{name}: {}", FieldTypeDisplay(ty))?;
        if let Some(default_value) = default_value {
            write!(f, " = {}", default_value)?;
        }
        Ok(())
    }
}

struct EnumValueDisplay<'a>(&'a EnumValue);

impl Display for EnumValueDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let EnumValue {
            name,
            description,
            deprecated,
        } = self.0;
        write!(f, "{}", DescriptionDisplay(description.as_deref()))?;
        write!(f, "{name}")?;
        writeln!(f, "{}", DeprecatedDisplay(deprecated))
    }
}

struct DeprecatedDisplay<'a>(&'a Deprecated);

impl Display for DeprecatedDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Deprecated::No => {}
            Deprecated::Yes(None) => write!(f, "@deprecated")?,
            Deprecated::Yes(Some(reason)) => write!(f, "@deprecated(reason: {reason})")?,
        }
        Ok(())
    }
}

struct SpecifiedByDisplay<'a>(Option<&'a str>);

impl Display for SpecifiedByDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            None => {}
            Some(url) => write!(f, " @specifiedBy(url: {url})")?,
        }
        Ok(())
    }
}

struct WrappingDisplay<'a>(&'a FieldWrapping, &'a str);

impl Display for WrappingDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let WrappingDisplay(wrapping, inner_text) = self;
        let wrapping_types = wrapping.into_iter().collect::<Vec<_>>();
        for wrapping_type in &wrapping_types {
            match wrapping_type {
                WrappingType::List => write!(f, "[")?,
                WrappingType::NonNull => {}
            }
        }
        write!(f, "{inner_text}")?;
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
