//! Implements topological sorting so we print our structs in a nice order.

use std::{collections::HashSet, hash::Hash, rc::Rc};

pub trait Vertex: Hash + Eq + Sized {
    fn adjacents(&self) -> Vec<Self>;
}

pub fn topological_sort<'a, V: Vertex + Clone + 'a>(
    vertices: impl Iterator<Item = &'a V>,
) -> Vec<V> {
    let mut visited = HashSet::new();
    let mut stack = Vec::new();

    for vertex in vertices {
        if visited.contains(vertex) {
            continue;
        }
        sort_impl(vertex.clone(), &mut visited, &mut stack);
    }

    // Our dependencies always go parent -> child and we want parents
    // printed first so we reverse the final sort before returning.
    stack.into_iter().rev().collect()
}

fn sort_impl<'a, V: Vertex + Clone>(vertex: V, visited: &mut HashSet<V>, stack: &mut Vec<V>) {
    visited.insert(vertex.clone());

    for adjacent_vertex in vertex.adjacents() {
        if visited.contains(&adjacent_vertex) {
            continue;
        }
        sort_impl(adjacent_vertex, visited, stack)
    }

    stack.push(vertex)
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

    impl Vertex for Rc<TestVertex> {
        fn adjacents(&self) -> Vec<Rc<TestVertex>> {
            self.adjacents.clone()
        }
    }

    #[test]
    #[allow(clippy::many_single_char_names)]
    fn test_sorting_works() {
        let a = TestVertex::new("a", &[]);
        let b = TestVertex::new("b", &[]);
        let c = TestVertex::new("c", &[&a, &b]);
        let d = TestVertex::new("d", &[&c]);
        let e = TestVertex::new("e", &[&d]);

        let array = vec![&a, &b, &c, &d, &e];
        let sorted = topological_sort(array.into_iter());

        assert_eq!(sorted, vec![e, d, c, b, a]);
    }
}
