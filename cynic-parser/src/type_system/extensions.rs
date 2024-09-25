use super::Description;

impl<'a> Description<'a> {
    pub fn to_cow(&self) -> Cow<'a, str> {
        self.description().to_cow()
    }

    pub fn raw_str(&self) -> &'a str {
        self.description().raw_str()
    }
}
