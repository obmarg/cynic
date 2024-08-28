use std::fmt::Display;

#[derive(Debug)]
pub struct Attributes {
    name: String,
    attrs: Vec<String>,
}

impl Attributes {
    pub fn new(name: impl Into<String>) -> Self {
        Attributes {
            name: name.into(),
            attrs: vec![],
        }
    }

    pub fn push(&mut self, attribute: impl Into<String>) {
        self.attrs.push(attribute.into());
    }
}

impl Display for Attributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.attrs.is_empty() {
            return Ok(());
        }

        let name = &self.name;

        write!(f, "#[{name}(")?;
        for (i, attr) in self.attrs.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{attr}")?;
        }
        writeln!(f, ")]")
    }
}
