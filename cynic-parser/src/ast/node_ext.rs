pub use super::*;

impl NameOwner for FragmentDef {
    fn name(&self) -> Option<Name> {
        self.fragment_name().and_then(|f| f.name())
    }
}

impl NameOwner for FragmentSpread {
    fn name(&self) -> Option<Name> {
        self.fragment_name().and_then(|f| f.name())
    }
}

impl Directive {
    pub fn arguments(&self) -> AstChildren<Argument> {
        self.argument_list()
            .map(|al| al.arguments())
            .unwrap_or_else(|| support::children(self.syntax()))
    }
}

impl FieldSelection {
    pub fn arguments(&self) -> AstChildren<Argument> {
        self.argument_list()
            .map(|al| al.arguments())
            .unwrap_or_else(|| support::children(self.syntax()))
    }
}

impl FragmentDef {
    // TODO: Not sure this belongs here...
    pub fn applies_to_type(&self, ty: &str) -> bool {
        let condition_type = self
            .type_condition()
            .and_then(|n| Some(n.named_type()?.name()?.to_string()));

        return condition_type.as_deref() == Some(ty);
    }
}
