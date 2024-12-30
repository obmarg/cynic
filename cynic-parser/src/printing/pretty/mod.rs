mod executable;
mod printer;
mod sorting;
mod type_system;
mod values;

use printer::PrettyOptions;
pub use printer::PrettyPrinter;

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
