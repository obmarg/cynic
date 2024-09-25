use std::borrow::Cow;

use super::Description;

impl<'a> Description<'a> {
    pub fn to_cow(&self) -> Cow<'a, str> {
        self.literal().to_cow()
    }

    pub fn raw_str(&self) -> &'a str {
        self.literal().raw_str()
    }
}
