use crate::{Span, buffer::Buffer};

/// A reader that supports lookahead functionality for parsing.
pub struct ParseStream {
    input: Buffer,
    pos: usize,
    end: usize,
}

impl ParseStream {
    pub fn new(input: Buffer) -> Self {
        let end = input.len();
        let mut stream = ParseStream { input, pos: 0, end };
        stream.skip_whitespace();

        stream
    }

    /// Cuts the stream at the specified end position.
    pub fn cut_at(&mut self, end: usize) {
        self.end = self.end.min(self.pos + end);
    }

    pub fn is_empty(&self) -> bool {
        self.pos >= self.end
    }

    pub(crate) fn buffer(&self) -> &Buffer {
        &self.input
    }

    /// Peeks at the next character in the stream without consuming it.
    pub fn fork(&self) -> Self {
        ParseStream {
            input: self.input.clone(),
            pos: self.pos,
            end: self.end,
        }
    }

    /// Returns the current position in the stream.
    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn span(&self) -> Span {
        Span::new(self.pos, self.end)
    }

    /// Consumes `n` bytes from the stream and returns them as a string slice.
    pub fn consume(&mut self, n: usize) -> &str {
        let len = n.min(self.end - self.pos);
        let start = self.pos;
        self.pos += len;
        &self.input[start..self.pos]
    }

    pub fn peek_char(&self) -> Option<char> {
        self.as_str().chars().next()
    }

    pub fn as_str(&self) -> &str {
        &self.input.as_str()[self.pos..self.end]
    }

    fn skip_whitespace(&mut self) {
        let s = self.as_str();
        let len = s.len();
        match s.find(|c: char| !c.is_whitespace()) {
            None => {
                self.consume(len);
            }
            Some(pos) => {
                self.consume(pos);
            }
        }
    }

    fn skip_comment(&mut self) -> bool {
        if self.as_str().starts_with("//") {
            // Single-line comment
            if let Some(pos) = self.as_str().find('\n') {
                self.consume(pos);
            } else {
                // Consume until the end of the stream
                self.consume(self.end - self.pos);
            }
            true
        } else if self.as_str().starts_with("/*") {
            // Multi-line comment
            if let Some(end_pos) = self.as_str().find("*/") {
                self.consume(end_pos + 2); // +2 to consume the closing */
            } else {
                // Unterminated comment, consume until the end of the stream
                self.consume(self.end - self.pos);
            }
            true
        } else {
            false
        }
    }

    /// Advances the stream past any whitespace characters and comments.
    pub fn skip_comments_and_whitespace(&mut self) {
        self.skip_whitespace();

        loop {
            if !self.skip_comment() {
                break;
            }

            self.skip_whitespace();
        }
    }
}
