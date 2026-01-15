use std::fmt::Display;

pub struct TreePath {
    stack: Vec<String>,
    len: usize,
}

impl TreePath {

    #[inline]
    pub fn new() -> Self {
        Self {
            stack: vec![String::with_capacity(32); 16],
            len: 0,
        }
    }

    pub fn push(&mut self, name: &str) {
        if self.len == self.stack.len() {
            self.stack.push(String::with_capacity(32));
        }

        self.stack[self.len].clear();
        self.stack[self.len].push_str(name);
        self.len += 1;
    }

    pub fn push_index(&mut self, index: usize) {
        use std::fmt::Write;

        if self.len == self.stack.len() {
            self.stack.push(String::with_capacity(32));
        }

        self.stack[self.len].clear();
        write!(self.stack[self.len], "{index}").unwrap();
        self.len += 1;
    }

    pub fn pop(&mut self) {
        if self.len > 0 {
            self.len -= 1;
        }
    }

}

impl Display for TreePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.len == 0 {
            write!(f, ".")
        } else {
            write!(f, "{}", self.stack[..self.len].join("."))
        }
    }
}
