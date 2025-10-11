mod executable;
mod printer;
mod sorting;
mod type_system;
mod values;

use pretty::{DocAllocator, Pretty};
use printer::PrettyOptions;
pub use printer::PrettyPrinter;

use crate::{
    common::{StringLiteral, StringLiteralKind},
    printing::escape_string,
};

struct NodeDisplay<T>(T, PrettyOptions);

impl<T> NodeDisplay<T> {
    pub fn with_node<N>(&self, node: N) -> NodeDisplay<N> {
        NodeDisplay(node, self.1)
    }

    pub fn mapper<N>(&self) -> impl Fn(N) -> NodeDisplay<N> {
        let options = self.1;
        move |node| NodeDisplay(node, options)
    }
}

type Allocator<'a> = pretty::Arena<'a>;

impl<'a> Pretty<'a, Allocator<'a>> for NodeDisplay<StringLiteral<'a>> {
    fn pretty(self, allocator: &'a Allocator<'a>) -> pretty::DocBuilder<'a, Allocator<'a>, ()> {
        match self.0.kind() {
            StringLiteralKind::String => allocator
                .text(escape_string(self.0.to_cow().as_ref()))
                .double_quotes(),
            StringLiteralKind::Block => allocator
                .text(self.0.raw_untrimmed_str())
                .double_quotes()
                .double_quotes()
                .double_quotes(),
        }
    }
}
