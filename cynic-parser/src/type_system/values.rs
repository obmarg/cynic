use super::TypeSystemId;

impl TypeSystemId for crate::values::ids::ValueId {
    type Reader<'a> = crate::values::Value<'a>;

    fn read(self, document: &super::TypeSystemDocument) -> Self::Reader<'_> {
        document.values.read(self)
    }
}

impl TypeSystemId for crate::values::ids::ConstValueId {
    type Reader<'a> = crate::values::ConstValue<'a>;

    fn read(self, document: &super::TypeSystemDocument) -> Self::Reader<'_> {
        document.values.read(self)
    }
}
