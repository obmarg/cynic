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
        let s = self.to_pascal_case();

        let mut buf = String::with_capacity(s.len());
        let mut chars = s.chars();

        if let Some(first_char) = chars.next() {
            buf.extend(first_char.to_lowercase());
        }

        buf.extend(chars);

        buf
    }

    fn to_pascal_case(&self) -> std::string::String {
        let mut buf = String::with_capacity(self.len());
        let mut first_char = true;
        let mut prev_is_upper = false;
        let mut prev_is_underscore = false;
        let mut chars = self.chars().peekable();
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
                prev_is_upper = false;
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
