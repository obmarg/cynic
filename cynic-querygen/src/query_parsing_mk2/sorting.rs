//! Implements topological sorting so we print our structs in a nice order.

use std::{collections::HashSet, hash::Hash, rc::Rc};

pub trait Vertex: Hash + Eq {
    fn adjacents(self: &Rc<Self>) -> Vec<Rc<Self>>;
}

pub fn topological_sort<V: Vertex>(vertices: impl Iterator<Item = Rc<V>>) -> Vec<Rc<V>> {
    let mut visited = HashSet::new();
    let mut stack = Vec::new();

    for vertex in vertices {
        if visited.contains(&vertex) {
            continue;
        }
        sort_impl(&vertex, &mut visited, &mut stack);
    }

    // Our dependencies always go parent -> child and we want parents
    // printed first so we reverse the final sort before returning.
    stack.reverse();

    stack
}

fn sort_impl<V: Vertex>(vertex: &Rc<V>, visited: &mut HashSet<Rc<V>>, stack: &mut Vec<Rc<V>>) {
    visited.insert(Rc::clone(vertex));

    for adjacent_vertex in vertex.adjacents() {
        if visited.contains(&adjacent_vertex) {
            continue;
        }
        sort_impl(&adjacent_vertex, visited, stack)
    }

    stack.push(Rc::clone(vertex))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Hash, PartialEq, Eq, Debug)]
    struct TestVertex {
        name: &'static str,
        adjacents: Vec<Rc<TestVertex>>,
    }

    impl TestVertex {
        fn new(name: &'static str, others: &[&Rc<TestVertex>]) -> Rc<TestVertex> {
            Rc::new(TestVertex {
                name,
                adjacents: others.iter().map(|v| Rc::clone(v)).collect(),
            })
        }
    }

    impl Vertex for TestVertex {
        fn adjacents(self: &Rc<Self>) -> Vec<Rc<TestVertex>> {
            self.adjacents.iter().map(Rc::clone).collect()
        }
    }

    #[test]
    fn test_sorting_works() {
        let a = TestVertex::new("a", &[]);
        let b = TestVertex::new("b", &[]);
        let c = TestVertex::new("c", &[&a, &b]);
        let d = TestVertex::new("d", &[&c]);
        let e = TestVertex::new("e", &[&d]);

        let array = [&a, &b, &c, &d, &e];
        let list = array.iter().map(|v| Rc::clone(v));

        assert_eq!(topological_sort(list), vec![e, d, c, b, a]);
    }
}
