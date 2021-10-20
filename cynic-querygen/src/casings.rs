use std::borrow::Cow;

pub trait CasingExt {
    fn to_snake_case(&self) -> String;
    fn to_camel_case(&self) -> String;
    fn to_pascal_case(&self) -> String;

    fn to_screaming_snake_case(&self) -> String {
        self.to_snake_case().to_uppercase()
    }
}

impl CasingExt for &str {
    // Specifically re-implementing this because the inflector impl
    // doesn't do the right thing with leading underscores
    fn to_snake_case(&self) -> String {
        let mut buf = String::with_capacity(self.len() * 2);

        // Setting this to true to avoid adding underscores at the beginning
        let mut prev_is_upper = true;
        for c in self.chars() {
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

    fn to_camel_case(&self) -> String {
        // Might be nice to not use inflector for this but cba writing it just now
        inflector::cases::camelcase::to_camel_case(self)
    }

    fn to_pascal_case(&self) -> std::string::String {
        // Might be nice to not use inflector for this but cba writing it just now
        inflector::cases::pascalcase::to_pascal_case(self)
    }
}

impl CasingExt for String {
    fn to_snake_case(&self) -> String {
        self.as_str().to_snake_case()
    }

    fn to_camel_case(&self) -> String {
        self.as_str().to_camel_case()
    }

    fn to_pascal_case(&self) -> String {
        self.as_str().to_pascal_case()
    }
}

impl CasingExt for Cow<'_, str> {
    fn to_snake_case(&self) -> String {
        self.as_ref().to_snake_case()
    }

    fn to_camel_case(&self) -> String {
        self.as_ref().to_camel_case()
    }

    fn to_pascal_case(&self) -> String {
        self.as_ref().to_pascal_case()
    }
}
