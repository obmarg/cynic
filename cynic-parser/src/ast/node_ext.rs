pub use super::*;

impl NameOwner for FragmentDef {
    fn name(&self) -> Option<Name> {
        self.fragment_name().and_then(|f| f.name())
    }
}
