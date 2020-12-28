use std::fmt;

pub struct Indented<'a, D: ?Sized> {
    inner: &'a mut D,
    level: usize,
    indent_required: bool,
}

/// Helper function for creating a default indenter
pub fn indented<D: ?Sized>(f: &mut D, level: usize) -> Indented<'_, D> {
    Indented {
        inner: f,
        level,
        indent_required: true,
    }
}

impl<T> fmt::Write for Indented<'_, T>
where
    T: fmt::Write + ?Sized,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for (ind, line) in s.split('\n').enumerate() {
            if ind > 0 {
                self.inner.write_char('\n')?;
                self.indent_required = true;
            }

            if self.indent_required && !line.is_empty() {
                write!(self.inner, "{:indent$}{}", "", line, indent = self.level)?;
                self.indent_required = false;
            } else if !line.is_empty() {
                write!(self.inner, "{}", line)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fmt::Write;

    #[test]
    fn test_simple() {
        let output = &mut String::new();
        let input = "Hello!";
        let expected = "    Hello!";

        write!(indented(output, 4), "{}", input).unwrap();

        assert_eq!(output, expected);
    }

    #[test]
    fn test_multi_line() {
        let output = &mut String::new();
        let input = "Hello!\nThere!";
        let expected = "    Hello!\n    There!";

        write!(indented(output, 4), "{}", input).unwrap();

        assert_eq!(output, expected);
    }

    #[test]
    fn test_multi_write() {
        let output = &mut String::new();
        let input = "Hello!\nThere!";
        let expected = "    Hello!\n    There!\n    Hello!\n    There!\n";

        writeln!(indented(output, 4), "{}", input).unwrap();
        writeln!(indented(output, 4), "{}", input).unwrap();

        assert_eq!(output, expected);
    }

    #[test]
    fn test_nested_indents() {
        let output = &mut String::new();
        let input = "Hello!\nThere!";

        let mut level1 = indented(output, 2);
        writeln!(level1, "{}", input).unwrap();

        let mut level2 = indented(&mut level1, 2);
        writeln!(level2, "{}", input).unwrap();

        assert_eq!(output, "  Hello!\n  There!\n    Hello!\n    There!\n")
    }
}
