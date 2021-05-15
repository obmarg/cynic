use super::{generated::tokens::*, AstToken};

impl Name {
    pub fn text(&self) -> &str {
        self.syntax().text()
    }
}

impl std::cmp::Ord for Name {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.syntax().text().cmp(other.syntax().text())
    }
}

impl std::cmp::PartialOrd for Name {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.syntax().text().partial_cmp(other.syntax().text())
    }
}
