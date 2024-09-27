use pretty::{DocAllocator, Pretty};

use crate::{printing::escape_string, values::Value};

use super::{brackets_and_maybe_indent, Allocator, NodeDisplay};

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<Value<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        match &self.0 {
            Value::Variable(variable) => allocator.text(format!("{variable}")),
            Value::Int(value) => allocator.text(format!("{value}")),
            Value::Float(value) => allocator.text(format!("{value}")),
            Value::String(string) => allocator
                .text(escape_string(string.value()))
                .double_quotes(),
            Value::Boolean(value) => allocator.text(format!("{value}")),
            Value::Null(_) => allocator.text("null"),
            Value::Enum(value) => allocator.text(value.name()),
            Value::List(list) if list.is_empty() => allocator.nil().brackets(),
            Value::List(list) => brackets_and_maybe_indent(
                allocator
                    .intersperse(
                        list.items().map(self.mapper()),
                        allocator.text(",").append(allocator.line()),
                    )
                    .group()
                    .enclose(allocator.line_(), allocator.line_()),
            ),
            Value::Object(object) if object.is_empty() => allocator.nil().braces(),
            Value::Object(object) => allocator
                .intersperse(
                    object.fields().map(|field| {
                        allocator
                            .text(field.name())
                            .append(allocator.text(":"))
                            .append(allocator.space())
                            .append(self.with_node(field.value()))
                    }),
                    allocator.text(",").append(allocator.space()),
                )
                .group()
                .enclose(allocator.softline(), allocator.softline())
                .braces(),
        }
    }
}
