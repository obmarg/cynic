use std::{borrow::Cow, fmt};

use super::Description;

impl<'a> Description<'a> {
    pub fn to_cow(&self) -> Cow<'a, str> {
        self.literal().to_cow()
    }

    pub fn raw_str(&self) -> &'a str {
        self.literal().raw_str()
    }
}

impl fmt::Display for Description<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.literal().fmt(f)
    }
}
