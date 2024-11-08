use cynic_parser::Span;

#[derive(Clone, Copy)]
pub struct EnumValue<'a> {
    name: &'a str,
    span: Option<Span>,
}

impl<'a> EnumValue<'a> {
    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn span(&self) -> Option<Span> {
        self.span
    }
}

impl PartialEq for EnumValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for EnumValue<'_> {}

impl std::fmt::Debug for EnumValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::fmt::Display for EnumValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
