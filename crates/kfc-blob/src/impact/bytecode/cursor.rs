pub struct Cursor<'a> {
    index: usize,
    content: &'a str,
    chars: std::str::Chars<'a>,
}

#[allow(dead_code)]
impl<'a> Cursor<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            index: 0,
            content,
            chars: content.chars(),
        }
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn as_str(&self) -> &'a str {
        self.chars.as_str()
    }

    #[inline]
    pub fn content(&self) -> &'a str {
        self.content
    }

    pub fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    pub fn first(&self) -> char {
        self.chars.clone().next().unwrap_or('\0')
    }

    pub fn next(&mut self) -> Option<char> {
        self.chars.next()
    }

    pub fn skip(&mut self) {
        self.next();
    }

    pub fn skip_n(&mut self, n: usize) {
        for _ in 0..n {
            self.next();
        }
    }

    pub fn slice(&mut self) -> &'a str {
        let start = self.index;
        let end = self.content.len() - self.chars.as_str().len();

        self.index = end;
        &self.content[start..end]
    }

    pub fn peek_slice(&self) -> &'a str {
        let start = self.index;
        let end = self.content.len() - self.chars.as_str().len();

        &self.content[start..end]
    }

}
