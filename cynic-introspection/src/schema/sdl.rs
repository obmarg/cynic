use std::fmt::{self, Display, Write};

use crate::{
    Deprecated, EnumType, EnumValue, Field, FieldType, InputObjectType, InputValue, InterfaceType,
    ObjectType, ScalarType, Type, UnionType,
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
        if self.should_should_schema_definition() {
            writeln!(f, "schema {{")?;
            let f2 = &mut indented(f);
            writeln!(f2, "query: {}", schema.query_type)?;
            if let Some(mutation_type) = &schema.mutation_type {
                writeln!(f2, "mutation: {}", mutation_type)?;
            }
            if let Some(subscription_type) = &schema.subscription_type {
                writeln!(f2, "subscription: {}", subscription_type)?;
            }
            writeln!(f, "}}")?;
        }

        for ty in &schema.types {
            let ty = TypeDisplay(ty);
            write!(f, "{ty}")?;
        }

        Ok(())
    }
}

impl SchemaDisplay<'_> {
    fn should_should_schema_definition(&self) -> bool {
        self.0.query_type != "Query"
            || self
                .0
                .mutation_type
                .as_ref()
                .map(|mutation_type| mutation_type != "Mutation")
                .unwrap_or_default()
            || self
                .0
                .subscription_type
                .as_ref()
                .map(|subscription_type| subscription_type != "Subscription")
                .unwrap_or_default()
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
                if fields.is_empty() {
                    writeln!(f, "type {name}{}", ImplementsDisplay(interfaces))
                } else {
                    writeln!(f, "type {name}{} {{", ImplementsDisplay(interfaces))?;
                    for field in fields {
                        writeln!(indented(f), "{}", FieldDisplay(field))?;
                    }
                    writeln!(f, "}}\n")
                }
            }
            Type::InputObject(InputObjectType {
                name,
                description,
                fields,
            }) => {
                write!(f, "{}", DescriptionDisplay(description.as_deref()))?;
                if fields.is_empty() {
                    writeln!(f, "input {name}")
                } else {
                    writeln!(f, "input {name} {{")?;
                    for field in fields {
                        writeln!(indented(f), "{}", InputValueDisplay(field))?;
                    }
                    writeln!(f, "}}\n")
                }
            }
            Type::Enum(EnumType {
                name,
                description,
                values,
            }) => {
                write!(f, "{}", DescriptionDisplay(description.as_deref()))?;
                if values.is_empty() {
                    writeln!(f, "enum {name}")
                } else {
                    writeln!(f, "enum {name} {{")?;
                    for value in values {
                        write!(indented(f), "{}", EnumValueDisplay(value))?;
                    }
                    writeln!(f, "}}\n")
                }
            }
            Type::Interface(InterfaceType {
                name,
                description,
                fields,
                interfaces,
                possible_types: _,
            }) => {
                write!(f, "{}", DescriptionDisplay(description.as_deref()))?;
                if fields.is_empty() {
                    writeln!(f, "interface {name}{}", ImplementsDisplay(interfaces))
                } else {
                    writeln!(f, "interface {name}{} {{", ImplementsDisplay(interfaces))?;
                    for field in fields {
                        writeln!(indented(f), "{}", FieldDisplay(field))?;
                    }
                    writeln!(f, "}}\n")
                }
            }
            Type::Union(union) => {
                write!(f, "{}", UnionDisplay(union))
            }
        }
    }
}

struct UnionDisplay<'a>(&'a UnionType);

impl Display for UnionDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let UnionType {
            name,
            description,
            possible_types,
        } = self.0;
        write!(f, "{}", DescriptionDisplay(description.as_deref()))?;
        let wrap = possible_types.iter().map(String::len).sum::<usize>() > 80;

        write!(f, "union {name} =")?;
        if wrap {
            write!(f, "\n   ")?;
        }
        for (i, ty) in possible_types.iter().enumerate() {
            if i != 0 {
                if wrap {
                    write!(f, "\n ")?;
                }
                write!(f, " |")?;
            }
            write!(f, " {ty}")?;
        }
        writeln!(f, "\n")
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
            let wrap = self.should_wrap_arguments();
            if wrap {
                writeln!(f, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i != 0 {
                        writeln!(f)?;
                    }
                    write!(indented(f), "{}", InputValueDisplay(arg))?;
                }
                write!(f, "\n)")?;
            } else {
                write!(f, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", InputValueDisplay(arg))?;
                }
                write!(f, ")")?;
            }
        }
        write!(
            f,
            ": {}{}",
            FieldTypeDisplay(ty),
            DeprecatedDisplay(deprecated)
        )?;

        Ok(())
    }
}

impl FieldDisplay<'_> {
    /// Hacky heuristic for whether we should wrap the field arguments
    fn should_wrap_arguments(&self) -> bool {
        if self.0.args.iter().any(|arg| arg.description.is_some()) {
            return true;
        }
        let arg_len = self
            .0
            .args
            .iter()
            .map(|arg| arg.name.len() + arg.ty.name.len())
            .sum::<usize>();
        arg_len + self.0.ty.name.len() > 80
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
        write!(f, "{}", self.0)
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
            Deprecated::Yes(None) => write!(f, " @deprecated")?,
            Deprecated::Yes(Some(reason)) => {
                write!(f, " @deprecated(reason: \"{}\")", escape_string(reason))?
            }
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

pub fn indented<D>(f: &mut D) -> indenter::Indented<'_, D> {
    indenter::indented(f).with_str("  ")
}

fn escape_string(src: &str) -> String {
    let mut dest = String::with_capacity(src.len());

    for character in src.chars() {
        match character {
            '"' | '\\' | '\n' | '\r' | '\t' => {
                dest.extend(character.escape_default());
            }
            _ => dest.push(character),
        }
    }

    dest
}
