use super::token::Span;

#[derive(Debug)]
pub struct ParseError {
    pub span: Span,
    pub kind: ParseErrorKind,
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} - {}", self.span, self.kind)
    }
}

#[derive(Debug)]
pub enum ParseErrorKind {
    DuplicateLabel {
        label: String,
    },
    UnknownLabel {
        label: String,
    },
    Expected {
        expected: String,
        found: String,
    },
    NumberParseError {
        content: String,
        error: std::num::ParseIntError,
    },
    UnknownType {
        type_name: String,
    },
    UnknownData {
        name: String,
    },
}

impl std::fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ParseErrorKind::DuplicateLabel { label } => {
                write!(f, "Duplicate label `{}`", label)
            }
            ParseErrorKind::UnknownLabel { label } => {
                write!(f, "Unknown label `{}`", label)
            }
            ParseErrorKind::Expected { expected, found } => {
                write!(f, "Expected {}, found {}", expected, found)
            }
            ParseErrorKind::NumberParseError { content, error } => {
                write!(f, "Failed to parse number `{}`: {}", content, error)
            }
            ParseErrorKind::UnknownType { type_name } => {
                write!(f, "Unknown type `{}`", type_name)
            }
            ParseErrorKind::UnknownData { name } => {
                write!(f, "Unknown data `{}`", name)
            }
        }
    }
}
