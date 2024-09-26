use super::ExecutableId;

impl ExecutableId for crate::values::ids::ValueId {
    type Reader<'a> = crate::values::Value<'a>;

    fn read(self, document: &super::ExecutableDocument) -> Self::Reader<'_> {
        document.values.read(self)
    }
}
