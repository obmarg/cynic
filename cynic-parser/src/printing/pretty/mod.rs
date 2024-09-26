mod executable;
mod printer;
mod sorting;
mod type_system;
mod values;

pub use printer::PrettyPrinter;

struct NodeDisplay<T>(T);

type Allocator<'a> = pretty::Arena<'a>;

fn brackets_and_maybe_indent<'a>(
    thing: pretty::DocBuilder<'a, Allocator<'a>>,
) -> pretty::DocBuilder<'a, Allocator<'a>> {
    thing.clone().nest(2).brackets().flat_alt(thing.brackets())
}
