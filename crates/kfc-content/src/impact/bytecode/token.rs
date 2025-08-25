use std::fmt::Display;

#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub content: &'a str,
    pub span: Span,
}

impl<'a> Token<'a> {
    pub fn new(
        kind: TokenKind,
        content: &'a str,
        span: Span
    ) -> Self {
        Self {
            kind,
            content,
            span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self {
            start,
            end,
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} - {}", self.start, self.end)
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub index: usize,
}

impl Position {
    pub fn new(line: usize, column: usize, index: usize) -> Self {
        Self {
            line,
            column,
            index,
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Comment,
    Whitespace,
    Newline,

    Identifier,
    Number,

    Eof,
    Unknown
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Comment => write!(f, "comment"),
            Self::Whitespace => write!(f, "whitespace"),
            Self::Newline => write!(f, "newline"),
            Self::Identifier => write!(f, "identifier"),
            Self::Number => write!(f, "number"),
            Self::Eof => write!(f, "EOF"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordKind {
    Invalid,
    IAdd,
    ISub,
    IMul,
    IDiv,
    Ilt,
    Ieq,
    Ileq,
    Br,
    Brt,
    Brf,
    IConst,
    IConst0,
    IConst1,
    Inc,
    Dec,
    Copy,
    Dup,
    Call,
    ECall,
    Ret,
    Load,
    GLoad,
    Store,
    GStore,
    LTime,
    TimeFF,
    Pop,
    Rvm,
    DSelf,
    Halt,
    Unknown,
}
