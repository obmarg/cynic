use std::fmt::{self, Write};

use crate::{common::OperationType, executable::*, printing::indent::indented};

impl fmt::Display for ExecutableDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let use_short_form = {
            let mut operation_iter = self.operations();
            let first_op = operation_iter.next();
            match (first_op, operation_iter.next()) {
                (Some(first_op), None) => {
                    first_op.name().is_none()
                        && first_op.description().is_none()
                        && first_op.operation_type() == OperationType::Query
                        && first_op.directives().len() == 0
                        && first_op.variable_definitions().len() == 0
                }
                _ => false,
            }
        };

        for (i, definition) in self.definitions().enumerate() {
            if i != 0 {
                writeln!(f)?;
            }
            match definition {
                ExecutableDefinition::Operation(op) if use_short_form => {
                    write!(f, "{}", op.selection_set())?;
                }
                ExecutableDefinition::Operation(op) => {
                    write!(f, "{op}")?;
                }
                ExecutableDefinition::Fragment(fragment) => {
                    write!(f, "{fragment}")?;
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for OperationDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = self.description() {
            write!(f, "{description}")?;
        }
        write!(f, "{}", self.operation_type())?;
        if let Some(name) = self.name() {
            write!(f, " {name}")?;
        }
        writeln!(
            f,
            "{}{} {}",
            self.variable_definitions(),
            self.directives(),
            self.selection_set()
        )
    }
}

impl fmt::Display for FragmentDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = self.description() {
            write!(f, "{description}")?;
        }
        writeln!(
            f,
            "fragment {} on {}{} {}",
            self.name(),
            self.type_condition(),
            self.directives(),
            self.selection_set()
        )
    }
}

impl<'a> fmt::Display for iter::Iter<'a, VariableDefinition<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.len() != 0 {
            write!(f, "(")?;
            for (i, definition) in self.clone().enumerate() {
                if i != 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{definition}")?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for VariableDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = self.description() {
            write!(f, "\n{description}")?;
        }
        write!(f, "${}: {}", self.name(), self.ty())?;

        if let Some(default) = self.default_value() {
            write!(f, " = {}", default)?;
        }

        write!(f, "{}", self.directives())
    }
}

impl<'a> std::fmt::Display for iter::Iter<'a, Selection<'a>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.len() != 0 {
            writeln!(f, "{{")?;
            for child in self.clone() {
                writeln!(indented(f, 2), "{}", child)?;
            }
            write!(f, "}}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Selection<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Selection::Field(field) => {
                write!(f, "{field}")
            }
            Selection::InlineFragment(fragment) => {
                write!(f, "{fragment}")
            }
            Selection::FragmentSpread(spread) => {
                write!(f, "{spread}")
            }
        }
    }
}

impl fmt::Display for FieldSelection<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(alias) = self.alias() {
            write!(f, "{}: ", alias)?;
        }

        let space = if self.selection_set().len() != 0 {
            " "
        } else {
            ""
        };

        write!(
            f,
            "{}{}{}{space}{}",
            self.name(),
            self.arguments(),
            self.directives(),
            self.selection_set()
        )
    }
}

impl fmt::Display for InlineFragment<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "...")?;

        if let Some(on_type) = self.type_condition() {
            write!(f, " on {}", on_type)?;
        }

        write!(f, "{} {}", self.directives(), self.selection_set())
    }
}

impl fmt::Display for FragmentSpread<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "... {}{}", self.fragment_name(), self.directives())
    }
}

impl<'a> fmt::Display for iter::Iter<'a, Directive<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for directive in self.clone() {
            write!(f, " {directive}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Directive<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}{}", self.name(), self.arguments())
    }
}

impl<'a> fmt::Display for iter::Iter<'a, Argument<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.len() != 0 {
            write!(f, "(")?;
            for (i, arg) in self.clone().enumerate() {
                if i != 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", arg)?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl fmt::Display for Argument<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name(), self.value())
    }
}

impl fmt::Display for Description<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.literal().kind() {
            crate::common::StringLiteralKind::String => {
                writeln!(f, "\"{}\"", self.literal().raw_untrimmed_str())
            }
            crate::common::StringLiteralKind::Block => {
                writeln!(f, "\"\"\"{}\"\"\"", self.literal().raw_untrimmed_str())
            }
        }
    }
}
