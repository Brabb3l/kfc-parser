use super::token::*;
use super::cursor::Cursor;

pub struct Tokenizer<'a> {
    cursor: Cursor<'a>,
    line: usize,
    column: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            cursor: Cursor::new(content),
            line: 1,
            column: 1,
        }
    }

    pub fn advance(&mut self) -> Token<'a> {
        let start = Position::new(self.line, self.column, self.cursor.index());
        let char = match self.next() {
            Some(c) => c,
            None => return Token::new(TokenKind::Eof, "", Span::new(start.clone(), start)),
        };

        let kind = match char {
            '#' => {
                self.skip_while(|c| c != '\n' && c != '\r');
                TokenKind::Comment
            }
            ' ' | '\t' => {
                self.skip_while(|c| c == ' ' || c == '\t');
                TokenKind::Whitespace
            }
            '\r' => {
                self.optional('\n');
                self.next_line();
                TokenKind::Newline
            }
            '\n' => {
                self.next_line();
                TokenKind::Newline
            }

            '0'..='9' => {
                self.skip_while(|c| c.is_numeric());
                
                let next = self.first();
                
                if next.is_ascii_alphanumeric() || next == '_' || next == ':' {
                    self.skip();
                    self.skip_while(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':');
                    TokenKind::Identifier
                } else {
                    TokenKind::Number
                }
            }
            
            'a'..='z' | 'A'..='Z' | '_' => {
                self.skip_while(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':');
                TokenKind::Identifier
            }

            _ => TokenKind::Unknown,
        };

        let content = self.cursor.slice();
        let end = Position::new(self.line, self.column, self.cursor.index());

        Token::new(kind, content, Span::new(start, end))
    }
    
    #[inline]
    fn first(&self) -> char {
        self.cursor.first()
    }
    
    #[inline]
    fn next(&mut self) -> Option<char> {
        self.column += 1;
        self.cursor.next()
    }
    
    #[inline]
    fn skip(&mut self) {
        self.column += 1;
        self.cursor.skip();
    }
    
    #[inline]
    fn skip_while<F>(&mut self, mut predicate: F)
    where
        F: FnMut(char) -> bool
    {
        while predicate(self.cursor.first()) && !self.cursor.is_eof() {
            self.skip();
        }
    }

    #[inline]
    fn next_line(&mut self) {
        self.line += 1;
        self.column = 1;
    }

    #[inline]
    fn optional(&mut self, c: char) -> bool {
        if self.first() == c {
            self.skip();
            true
        } else {
            false
        }
    }

}
