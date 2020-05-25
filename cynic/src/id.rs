pub struct Id(String);

impl Id {
    fn inner(&self) -> &str {
        return &self.0;
    }

    fn into_inner(self) -> String {
        return self.0;
    }
}

impl<T: Into<String>> From<T> for Id {
    fn from(s: T) -> Id {
        Id(s.into())
    }
}
