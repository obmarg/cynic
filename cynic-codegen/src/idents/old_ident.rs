#[derive(Debug, Clone, Copy)]
/// Rules to rename all fields in an InputObject or variants in an Enum
/// as GraphQL naming conventions usually don't match rust
pub enum RenameAll {
    None,
    /// For names that are entirely lowercase in GraphQL: `myfield`
    Lowercase,
    /// For names that are entirely uppercase in GraphQL: `MYFIELD`
    Uppercase,
    /// For names that are entirely pascal case in GraphQL: `MyField`
    PascalCase,
    /// For names that are entirely camel case in GraphQL: `myField`
    CamelCase,
    /// For names that are entirely snake case in GraphQL: `my_field`
    SnakeCase,
    /// For names that are entirely snake case in GraphQL: `MY_FIELD`
    ScreamingSnakeCase,
}

impl RenameAll {
    pub(super) fn apply(&self, string: impl AsRef<str>) -> String {
        match self {
            RenameAll::Lowercase => string.as_ref().to_lowercase(),
            RenameAll::Uppercase => string.as_ref().to_uppercase(),
            RenameAll::PascalCase => to_pascal_case(string.as_ref()),
            RenameAll::CamelCase => to_camel_case(string.as_ref()),
            RenameAll::SnakeCase => to_snake_case(string.as_ref()),
            RenameAll::ScreamingSnakeCase => to_snake_case(string.as_ref()).to_uppercase(),
            RenameAll::None => string.as_ref().to_string(),
        }
    }
}

impl darling::FromMeta for RenameAll {
    fn from_string(value: &str) -> Result<RenameAll, darling::Error> {
        match value.to_lowercase().as_ref() {
            "none" => Ok(RenameAll::None),
            "lowercase" => Ok(RenameAll::Lowercase),
            "uppercase" => Ok(RenameAll::Uppercase),
            "pascalcase" => Ok(RenameAll::PascalCase),
            "camelcase" => Ok(RenameAll::CamelCase),
            "snake_case" => Ok(RenameAll::SnakeCase),
            "screaming_snake_case" => Ok(RenameAll::ScreamingSnakeCase),
            _ => {
                // Feels like it'd be nice if this error listed all the options...
                Err(darling::Error::unknown_value(value))
            }
        }
    }
}

pub fn to_snake_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    // Setting this to true to avoid adding underscores at the beginning
    let mut prev_is_upper = true;
    for c in s.chars() {
        if c.is_uppercase() && !prev_is_upper {
            buf.push('_');
            buf.extend(c.to_lowercase());
            prev_is_upper = true;
        } else if c.is_uppercase() {
            buf.extend(c.to_lowercase());
        } else {
            prev_is_upper = false;
            buf.push(c);
        }
    }
    buf
}

// TODO: move this somewhere else...
pub fn to_pascal_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut first_char = true;
    let mut prev_is_upper = false;
    let mut prev_is_underscore = false;
    let mut chars = s.chars().peekable();
    loop {
        let c = chars.next();
        if c.is_none() {
            break;
        }
        let c = c.unwrap();
        if first_char {
            if c == '_' {
                // keep leading underscores
                buf.push('_');
                while let Some('_') = chars.peek() {
                    buf.push(chars.next().unwrap());
                }
            } else if c.is_uppercase() {
                prev_is_upper = true;
                buf.push(c);
            } else {
                buf.extend(c.to_uppercase());
            }
            first_char = false;
            continue;
        }

        if c.is_uppercase() {
            if prev_is_upper {
                buf.extend(c.to_lowercase());
            } else {
                buf.push(c);
            }
            prev_is_upper = true;
        } else if c == '_' {
            prev_is_underscore = true;
        } else {
            if prev_is_upper {
                buf.extend(c.to_lowercase())
            } else if prev_is_underscore {
                buf.extend(c.to_uppercase());
            } else {
                buf.push(c);
            }
            prev_is_upper = false;
            prev_is_underscore = false;
        }
    }

    buf
}

pub(super) fn to_camel_case(s: &str) -> String {
    let s = to_pascal_case(s);

    let mut buf = String::with_capacity(s.len());
    let mut chars = s.chars();

    if let Some(first_char) = chars.next() {
        buf.extend(first_char.to_lowercase());
    }

    buf.extend(chars);

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_underscore() {
        assert_eq!(to_snake_case("_hello"), "_hello");
        assert_eq!(to_snake_case("_"), "_");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("aString"), "a_string");
        assert_eq!(to_snake_case("MyString"), "my_string");
        assert_eq!(to_snake_case("my_string"), "my_string");
        assert_eq!(to_snake_case("_another_one"), "_another_one");
        assert_eq!(to_snake_case("RepeatedUPPERCASE"), "repeated_uppercase");
        assert_eq!(to_snake_case("UUID"), "uuid");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("aString"), "aString");
        assert_eq!(to_camel_case("MyString"), "myString");
        assert_eq!(to_camel_case("my_string"), "myString");
        assert_eq!(to_camel_case("_another_one"), "_anotherOne");
        assert_eq!(to_camel_case("RepeatedUPPERCASE"), "repeatedUppercase");
        assert_eq!(to_camel_case("UUID"), "uuid");
        assert_eq!(to_camel_case("__typename"), "__typename");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("aString"), "AString");
        assert_eq!(to_pascal_case("MyString"), "MyString");
        assert_eq!(to_pascal_case("my_string"), "MyString");
        assert_eq!(to_pascal_case("_another_one"), "_anotherOne");
        assert_eq!(to_pascal_case("RepeatedUPPERCASE"), "RepeatedUppercase");
        assert_eq!(to_pascal_case("UUID"), "Uuid");
        assert_eq!(to_pascal_case("__typename"), "__typename");
    }

    #[test]
    fn casings_are_not_lossy_where_possible() {
        for s in ["snake_case_thing", "snake"] {
            assert_eq!(to_snake_case(&to_pascal_case(s)), s);
        }

        for s in ["PascalCase", "Pascal"] {
            assert_eq!(to_pascal_case(&to_snake_case(s)), s);
        }

        for s in ["camelCase", "camel"] {
            assert_eq!(to_camel_case(&to_snake_case(s)), s);
        }
    }
}
