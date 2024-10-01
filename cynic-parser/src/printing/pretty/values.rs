use pretty::{DocAllocator, Pretty};

use crate::{
    printing::escape_string,
    values::{ConstValue, Value},
};

use super::{Allocator, NodeDisplay};

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<Value<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        match &self.0 {
            Value::Variable(variable) => allocator.text(variable.to_string()),
            Value::Int(value) => allocator.text(value.to_string()),
            Value::Float(value) => allocator.text(value.to_string()),
            Value::String(value) => allocator.text(escape_string(value.value())).double_quotes(),
            Value::Boolean(value) => allocator.text(value.to_string()),
            Value::Null(_) => allocator.text("null"),
            Value::Enum(value) => allocator.text(value.name()),
            Value::List(list) if list.is_empty() => allocator.nil().brackets(),
            Value::List(list) => allocator
                .line_()
                .append(
                    allocator.intersperse(
                        list.items().map(self.mapper()),
                        allocator
                            .line_()
                            .append(allocator.nil().flat_alt(allocator.text(", "))),
                    ),
                )
                .nest(2)
                .append(allocator.line_())
                .brackets()
                .group(),
            Value::Object(object) if object.is_empty() => allocator.nil().braces(),
            Value::Object(object) => allocator
                .line()
                .append(
                    allocator.intersperse(
                        object.fields().map(|field| {
                            allocator
                                .text(field.name())
                                .append(allocator.text(":"))
                                .append(allocator.space())
                                .append(self.with_node(field.value()))
                        }),
                        allocator
                            .line_()
                            .append(allocator.nil().flat_alt(allocator.text(", "))),
                    ),
                )
                .nest(2)
                .append(allocator.line())
                .braces()
                .group(),
        }
    }
}

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<ConstValue<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        self.with_node(Value::from(self.0)).pretty(allocator)
    }
}
