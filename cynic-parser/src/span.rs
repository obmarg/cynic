#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Returns `true` if this [Span] overlaps with the other [Span].
    ///
    /// Will always return `false` when one of the spans is empty or invalid (`end < start`).
    pub fn overlaps(&self, other: Span) -> bool {
        self.start < other.end && other.start < self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn assert_overlap(a: Span, b: Span) {
        let forward = a.overlaps(b);
        let back = b.overlaps(a);
        assert_eq!(forward, back);

        assert!(
            forward,
            "Expected {a:?} and {b:?} to overlap but they don't."
        );
    }

    #[track_caller]
    fn assert_no_overlap(a: Span, b: Span) {
        let forward = a.overlaps(b);
        let back = b.overlaps(a);
        assert_eq!(forward, back);

        assert!(
            !forward,
            "Expected {a:?} and {b:?} not to overlap but they do."
        );
    }

    #[test]
    fn empty_spans_never_overlap() {
        assert_no_overlap(Span { start: 0, end: 0 }, Span { start: 0, end: 0 });
        assert_no_overlap(Span { start: 0, end: 0 }, Span { start: 10, end: 10 });
        assert_no_overlap(Span { start: 100, end: 0 }, Span { start: 10, end: 10 });
    }

    #[test]
    fn overlap_tests() {
        assert_overlap(Span { start: 0, end: 10 }, Span { start: 5, end: 15 });
        assert_overlap(Span { start: 5, end: 15 }, Span { start: 0, end: 10 });
        assert_overlap(Span { start: 0, end: 10 }, Span { start: 0, end: 10 });
        assert_overlap(Span { start: 0, end: 10 }, Span { start: 0, end: 5 });
        assert_overlap(Span { start: 0, end: 10 }, Span { start: 5, end: 10 });

        assert_no_overlap(Span { start: 10, end: 20 }, Span { start: 20, end: 30 });
        assert_no_overlap(Span { start: 10, end: 20 }, Span { start: 30, end: 40 });
    }
}
