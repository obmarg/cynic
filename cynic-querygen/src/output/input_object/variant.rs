use crate::schema::TypeSpec;

pub struct Variant<'a> {
    name: &'a str,
    rename: Option<&'a str>,
    type_spec: &'a TypeSpec<'a>,
}

impl<'a> Variant<'a> {
    pub fn new(name: &'a str, type_spec: &'a TypeSpec<'a>) -> Self {
        Variant {
            name,
            type_spec,
            rename: None,
        }
    }

    pub fn add_rename(&mut self, name: &'a str) {
        self.rename = Some(name);
    }

    fn rename(&self) -> Option<&'a str> {
        if let Some(rename) = self.rename {
            return Some(rename);
        }

        None
    }
}

impl std::fmt::Display for Variant<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(rename) = self.rename() {
            writeln!(f, r#"#[cynic(rename = "{}")]"#, rename)?;
        }
        writeln!(f, "{}({}),", &self.name, self.type_spec.name)
    }
}
