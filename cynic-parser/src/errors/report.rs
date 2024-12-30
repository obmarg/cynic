use core::fmt;

use ariadne::{Config, Label, ReportKind, Source};

use crate::Error;

pub struct Report<'doc> {
    inner: ariadne::Report<'static>,
    document: &'doc str,
}

impl Error {
    pub fn to_report<'a>(&self, document: &'a str) -> Report<'a> {
        let (message, label, note) = self.components();

        let mut builder = ariadne::Report::build(ReportKind::Error, (), 0)
            .with_message(message)
            .with_config(Config::default().with_color(false));

        if let Some(label) = label {
            builder.add_label(label);
        }

        if let Some(note) = note {
            builder.set_note(note)
        }

        let inner = builder.finish();

        Report { inner, document }
    }

    fn components(&self) -> (String, Option<Label>, Option<String>) {
        match self {
            Error::InvalidToken { location } => (
                "invalid token".into(),
                Label::new(*location..*location)
                    .with_message("could not understand this token")
                    .into(),
                None,
            ),
            Error::UnrecognizedEof { location, expected } => (
                "unexpected eof".into(),
                Label::new(*location..*location)
                    .with_message("expected another token here")
                    .into(),
                Some(format!("expected one of {}", expected.join(", "))),
            ),
            Error::UnrecognizedToken {
                token: (start, token, end),
                expected,
            } => (
                format!("unexpected {}", token),
                Label::new(*start..*end)
                    .with_message("didn't expect to see this")
                    .into(),
                Some(format!("expected one of {}", expected.join(", "))),
            ),
            Error::ExtraToken {
                token: (start, token, end),
            } => (
                format!("extra {}", token),
                Label::new(*start..*end)
                    .with_message("we expected the document to end here")
                    .into(),
                None,
            ),
            Error::Lexical(error) => {
                let span = error.span();
                (
                    "invalid token".into(),
                    Label::new(span.start..span.end)
                        .with_message("could not parse a token here")
                        .into(),
                    None,
                )
            }
            Error::MalformedStringLiteral(error) => {
                let span = self.span().unwrap();
                (
                    error.to_string(),
                    Label::new(span.start..span.end)
                        .with_message("error occurred here")
                        .into(),
                    None,
                )
            }
            Error::MalformedDirectiveLocation(_, _, _) => {
                let span = self.span().unwrap();
                (
                    self.to_string(),
                    Label::new(span.start..span.end)
                        .with_message("this is not a valid directive location")
                        .into(),
                    None,
                )
            }

            Error::VariableInConstPosition(_, _, _) => {
                let span = self.span().unwrap();
                (
                    self.to_string(),
                    Label::new(span.start..span.end)
                        .with_message("only non-variable values can be used here")
                        .into(),
                    None,
                )
            }
            Error::EmptyExecutableDocument => (self.to_string(), None, Some("graphql documents should contain at least one query, mutation or subscription".into())),
            Error::EmptyTypeSystemDocument => (self.to_string(), None, Some("graphql documents should contain at least one type, schema or directive definition".into())),
        }
    }
}

impl std::error::Error for Report<'_> {}

impl fmt::Display for Report<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = Vec::<u8>::new();
        self.inner
            .write(Source::from(self.document), &mut output)
            .unwrap();
        let s = String::from_utf8_lossy(&output);

        write!(f, "{s}")
    }
}

impl fmt::Debug for Report<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
