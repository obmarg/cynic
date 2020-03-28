use proc_macro2::Span;

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    IoError(String),
    ParseError(String),
}

impl Error {
    pub fn to_syn_error(self, schema_span: Span) -> syn::Error {
        let message = match self {
            Error::IoError(e) => format!("Could not load schema file: {}", e),
            Error::ParseError(e) => format!("Could not parse schema file: {}", e),
        };

        syn::Error::new(schema_span, message)
    }
}

impl From<graphql_parser::schema::ParseError> for Error {
    fn from(e: graphql_parser::schema::ParseError) -> Error {
        Error::ParseError(e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IoError(e.to_string())
    }
}
