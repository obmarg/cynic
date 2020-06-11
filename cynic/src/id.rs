#[derive(Clone, Debug)]
pub struct Id(String);

impl Id {
    pub fn inner(&self) -> &str {
        return &self.0;
    }

    pub fn into_inner(self) -> String {
        return self.0;
    }
}

impl<T: Into<String>> From<T> for Id {
    fn from(s: T) -> Id {
        Id(s.into())
    }
}
