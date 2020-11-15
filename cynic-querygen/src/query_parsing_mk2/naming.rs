use std::{collections::HashMap, hash::Hash, rc::Rc};

pub trait Nameable: Eq + Hash {
    fn requested_name(&self) -> String;
}

#[derive(Debug)]
pub struct Namer<Subject> {
    named_subjects: HashMap<Subject, String>,
    used_names: HashMap<String, u16>,
}

impl<Subject> Namer<Subject>
where
    Subject: Nameable + Clone,
{
    pub fn new() -> Namer<Subject> {
        Namer {
            named_subjects: HashMap::new(),
            used_names: HashMap::new(),
        }
    }

    pub fn force_name(&mut self, subject: &Subject, name: impl Into<String>) -> String {
        self.impl_naming(subject, name.into())
    }

    pub fn name_subject(&mut self, subject: &Subject) -> String {
        if let Some(name) = self.named_subjects.get(subject) {
            return name.clone();
        }

        self.impl_naming(subject, subject.requested_name())
    }

    fn impl_naming(&mut self, subject: &Subject, requested_name: String) -> String {
        let used_count = self.used_names.entry(requested_name.clone()).or_insert(0);
        *used_count += 1;
        let name = if *used_count == 1 {
            requested_name.to_string()
        } else {
            format!("{}{}", requested_name, used_count)
        };

        self.named_subjects.insert(subject.clone(), name.clone());

        name
    }
}

impl<'query, 'schema> Nameable for Rc<super::normalisation::SelectionSet<'query, 'schema>> {
    fn requested_name(&self) -> String {
        self.target_type.name().to_owned()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn write_some_tests() {
        todo!()
    }
}
