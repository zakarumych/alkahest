use std::rc::Rc;

use crate::{Span, parse::ParseStream};

#[derive(Clone, Copy, Debug)]
pub enum LexErrorKind {
    UnexpectedEndOfInput,
    UnexpectedCharacter(char),
    UnclosedDelimiter,
}

#[derive(Clone, Copy, Debug)]
pub struct LexError {
    span: Span,
    kind: LexErrorKind,
}

impl LexError {
    pub fn span(&self) -> Span {
        self.span
    }

    pub fn kind(&self) -> LexErrorKind {
        self.kind
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spacing {
    /// Punctuation is joined with the next token (no space).
    Joint,
    /// Punctuation is separated from the next token by whitespace.
    Alone,
}

#[derive(Clone)]
pub struct Punct {
    ch: char,
    spacing: Spacing,
    span: Span,
}

impl Punct {
    #[inline]
    pub fn new(ch: char, spacing: Spacing, span: Span) -> Self {
        Punct { ch, spacing, span }
    }

    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }

    #[inline]
    pub fn char(&self) -> char {
        self.ch
    }

    #[inline]
    pub(crate) fn eq_str(&self, s: &str) -> bool {
        let mut buf = [0; 4];
        self.ch.encode_utf8(&mut buf) == s
    }

    #[inline]
    pub fn spacing(&self) -> Spacing {
        self.spacing
    }

    #[inline]
    pub fn is_punct(stream: &ParseStream) -> bool {
        match stream.peek_char() {
            Some(c) => Self::is_punct_ch(c),
            None => false,
        }
    }

    #[inline]
    fn is_punct_ch(ch: char) -> bool {
        matches!(
            ch,
            '!' | '#'
                | '$'
                | '%'
                | '&'
                | '\''
                | '*'
                | '+'
                | ','
                | '-'
                | '.'
                | '/'
                | ':'
                | ';'
                | '<'
                | '='
                | '>'
                | '?'
                | '@'
                | '\\'
                | '^'
                | '_'
                | '`'
                | '|'
                | '~'
        )
    }

    pub fn parse(stream: &mut ParseStream) -> Result<Self, LexError> {
        let start_pos = stream.pos();
        let ch = stream.peek_char().ok_or(LexError {
            span: Span::new(start_pos, start_pos),
            kind: LexErrorKind::UnexpectedEndOfInput,
        })?;

        if !Self::is_punct_ch(ch) {
            return Err(LexError {
                span: Span::new(start_pos, start_pos + ch.len_utf8()),
                kind: LexErrorKind::UnexpectedCharacter(ch),
            });
        }

        stream.consume(ch.len_utf8());

        let spacing = match stream.peek_char() {
            Some(ch) if Self::is_punct_ch(ch) => Spacing::Joint,
            _ => Spacing::Alone,
        };

        let end_pos = stream.pos();

        Ok(Punct::new(ch, spacing, Span::new(start_pos, end_pos)))
    }
}

pub struct Ident {
    string: Rc<str>,
    span: Span,
}

impl Ident {
    #[inline]
    pub fn new(string: Rc<str>, span: Span) -> Self {
        Ident { string, span }
    }

    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }

    pub fn as_str(&self) -> &str {
        &*self.string
    }

    pub(crate) fn clone_rc_str(&self) -> Rc<str> {
        self.string.clone()
    }

    #[inline]
    pub fn is_ident(stream: &ParseStream) -> bool {
        match stream.peek_char() {
            Some(c) => unicode_ident::is_xid_start(c),
            None => false,
        }
    }

    pub fn parse(stream: &mut ParseStream) -> Result<Self, LexError> {
        let start_pos = stream.pos();
        let mut end_pos = start_pos;

        let s = stream.buffer().as_str();

        let mut chars = s[start_pos..].chars();

        let first_char = chars.next().ok_or(LexError {
            span: Span::new(start_pos, start_pos),
            kind: LexErrorKind::UnexpectedEndOfInput,
        })?;

        if !unicode_ident::is_xid_start(first_char) {
            return Err(LexError {
                span: Span::new(start_pos, start_pos + first_char.len_utf8()),
                kind: LexErrorKind::UnexpectedCharacter(first_char),
            });
        }

        end_pos += first_char.len_utf8();

        for ch in chars {
            if !unicode_ident::is_xid_continue(ch) {
                break;
            }
            end_pos += ch.len_utf8();
        }

        let ident_str = &s[start_pos..end_pos];
        let ident = Rc::from(ident_str);

        stream.consume(end_pos - start_pos);

        Ok(Ident::new(ident, Span::new(start_pos, end_pos)))
    }
}

enum LiteralKind {
    Number,
    Char,
    String,
    Bytes,
}

pub struct Literal {
    value: Rc<str>,
    kind: LiteralKind,
    span: Span,
}

impl Literal {
    #[inline]
    pub fn new(value: Rc<str>, kind: LiteralKind, span: Span) -> Self {
        Literal { value, kind, span }
    }

    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }

    pub fn as_str(&self) -> &str {
        &*self.value
    }

    pub(crate) fn clone_rc_str(&self) -> Rc<str> {
        self.value.clone()
    }

    #[inline]
    fn is_literal(stream: &ParseStream) -> bool {
        match stream.peek_char() {
            Some(c) => c.is_ascii_digit() || c == '"' || c == '\'' || c == 'b',
            None => false,
        }
    }

    fn parse(stream: &mut ParseStream) -> Result<Self, LexError> {
        let start_pos = stream.pos();

        match stream.peek_char() {
            Some(c) if c.is_ascii_digit() => {
                // Parse number literal
                let mut end_pos = start_pos;
                let mut has_point = false;
                let mut has_exponent = false;

                let s = stream.buffer().as_str();
                let chars = s[start_pos..].chars();

                for ch in chars {
                    if ch.is_ascii_digit() || ch == '_' {
                        end_pos += ch.len_utf8();
                        continue;
                    }

                    if !has_point && ch == '.' {
                        has_point = true;
                        end_pos += ch.len_utf8();
                        continue;
                    }

                    if !has_exponent && (ch == 'e' || ch == 'E') {
                        has_exponent = true;
                        end_pos += ch.len_utf8();
                        continue;
                    }

                    break;
                }

                let literal_str = &s[start_pos..end_pos];
                let literal = Rc::from(literal_str);

                stream.consume(end_pos - start_pos);

                Ok(Literal::new(
                    literal,
                    LiteralKind::Number,
                    Span::new(start_pos, end_pos),
                ))
            }
            Some('"') => {
                // Parse string literal
                let mut end_pos = start_pos + 1; // Skip opening quote

                let s = stream.buffer().as_str();
                let mut chars = s[end_pos..].chars();

                while let Some(ch) = chars.next() {
                    end_pos += ch.len_utf8();
                    if ch == '"' {
                        break;
                    }
                    if ch == '\\' {
                        // Skip escaped character
                        if let Some(esc_ch) = chars.next() {
                            end_pos += esc_ch.len_utf8();
                        }
                    }
                }

                let literal_str = &s[start_pos..end_pos];
                let literal = Rc::from(literal_str);

                stream.consume(end_pos - start_pos);

                Ok(Literal::new(
                    literal,
                    LiteralKind::String,
                    Span::new(start_pos, end_pos),
                ))
            }

            Some(c) if c == '\'' => {
                // Parse char literal
                let mut end_pos = start_pos + 1; // Skip opening quote

                let s = stream.buffer().as_str();
                let mut chars = s[end_pos..].chars();

                while let Some(ch) = chars.next() {
                    end_pos += ch.len_utf8();
                    if ch == '\'' {
                        break;
                    }
                    if ch == '\\' {
                        // Skip escaped character
                        if let Some(esc_ch) = chars.next() {
                            end_pos += esc_ch.len_utf8();
                        }
                    }
                }

                let literal_str = &s[start_pos..end_pos];
                let literal = Rc::from(literal_str);

                stream.consume(end_pos - start_pos);

                Ok(Literal::new(
                    literal,
                    LiteralKind::Char,
                    Span::new(start_pos, end_pos),
                ))
            }
            Some(c) => Err(LexError {
                span: Span::new(start_pos, start_pos + c.len_utf8()),
                kind: LexErrorKind::UnexpectedCharacter(c),
            }),
            None => Err(LexError {
                span: Span::new(start_pos, start_pos),
                kind: LexErrorKind::UnexpectedEndOfInput,
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delimiter {
    Parenthesis,
    Brace,
    Bracket,
}

pub struct Group {
    delimiter: Delimiter,
    stream: TokenStream,
    span: Span,
}

impl Group {
    #[inline]
    pub fn new(delimiter: Delimiter, stream: TokenStream, span: Span) -> Self {
        Group {
            delimiter,
            stream,
            span,
        }
    }

    #[inline]
    pub fn delimiter(&self) -> Delimiter {
        self.delimiter
    }

    #[inline]
    pub fn stream(&self) -> TokenStream {
        self.stream.fork()
    }

    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }

    #[inline]
    fn is_group(stream: &ParseStream) -> bool {
        match stream.peek_char() {
            Some('(') | Some('{') | Some('[') => true,
            _ => false,
        }
    }

    fn parse(stream: &mut ParseStream) -> Result<Self, LexError> {
        let start_pos = stream.pos();

        // Consume opening delimiter
        let open_delim = match stream.peek_char() {
            Some('(') => Delimiter::Parenthesis,
            Some('{') => Delimiter::Brace,
            Some('[') => Delimiter::Bracket,
            Some(c) => {
                return Err(LexError {
                    span: Span::new(start_pos, start_pos + c.len_utf8()),
                    kind: LexErrorKind::UnexpectedCharacter(c),
                });
            }
            None => {
                return Err(LexError {
                    span: Span::new(start_pos, start_pos),
                    kind: LexErrorKind::UnexpectedEndOfInput,
                });
            }
        };

        stream.consume(1);

        // Find matching closing delimiter

        let mut stack = vec![];

        let s = stream.as_str();

        for (at, d) in s.match_indices(|ch: char| matches!(ch, '(' | ')' | '{' | '}' | '[' | ']')) {
            match d {
                "(" => stack.push(Delimiter::Parenthesis),
                "{" => stack.push(Delimiter::Brace),
                "[" => stack.push(Delimiter::Bracket),
                ")" | "]" | "}" => {
                    let close_delim = match d {
                        ")" => Delimiter::Parenthesis,
                        "}" => Delimiter::Brace,
                        "]" => Delimiter::Bracket,
                        _ => unreachable!(),
                    };

                    match stack.pop() {
                        None => {
                            if open_delim != close_delim {
                                return Err(LexError {
                                    span: Span::new(start_pos, start_pos + at + 2),
                                    kind: LexErrorKind::UnclosedDelimiter,
                                });
                            }

                            let mut group_stream = stream.fork();
                            group_stream.cut_at(at);
                            stream.consume(at + 1);

                            return Ok(Group::new(
                                open_delim,
                                TokenStream {
                                    inner: group_stream,
                                },
                                Span::new(start_pos, stream.pos()),
                            ));
                        }
                        Some(inner_delim) => {
                            if inner_delim != close_delim {
                                return Err(LexError {
                                    span: Span::new(start_pos, start_pos + at + 2),
                                    kind: LexErrorKind::UnclosedDelimiter,
                                });
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }

        Err(LexError {
            span: Span::new(start_pos, start_pos + s.len()),
            kind: LexErrorKind::UnclosedDelimiter,
        })
    }
}

pub enum Token {
    Punct(Punct),
    Ident(Ident),
    Literal(Literal),
    Group(Group),
}

impl Token {
    pub fn span(&self) -> Span {
        match self {
            Token::Punct(p) => p.span,
            Token::Ident(i) => i.span,
            Token::Literal(l) => l.span,
            Token::Group(g) => g.span,
        }
    }
}

pub struct TokenStream {
    inner: ParseStream,
}

impl TokenStream {
    #[inline]
    pub fn new(stream: ParseStream) -> Self {
        let mut stream = stream;
        stream.skip_comments_and_whitespace();
        TokenStream { inner: stream }
    }

    #[inline]
    pub fn fork(&self) -> TokenStream {
        TokenStream {
            inner: self.inner.fork(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span()
    }

    #[inline]
    pub fn next(&mut self) -> Result<Token, LexError> {
        if self.inner.is_empty() {
            return Err(LexError {
                span: Span::new(self.inner.pos(), self.inner.pos()),
                kind: LexErrorKind::UnexpectedEndOfInput,
            });
        }

        if Punct::is_punct(&self.inner) {
            let punct = Punct::parse(&mut self.inner)?;
            self.inner.skip_comments_and_whitespace();
            return Ok(Token::Punct(punct));
        }

        if Ident::is_ident(&self.inner) {
            let ident = Ident::parse(&mut self.inner)?;
            self.inner.skip_comments_and_whitespace();
            return Ok(Token::Ident(ident));
        }

        if Literal::is_literal(&self.inner) {
            let literal = Literal::parse(&mut self.inner)?;
            self.inner.skip_comments_and_whitespace();
            return Ok(Token::Literal(literal));
        }

        if Group::is_group(&self.inner) {
            let group = Group::parse(&mut self.inner)?;
            self.inner.skip_comments_and_whitespace();
            return Ok(Token::Group(group));
        }

        let pos = self.inner.pos();
        match self.inner.peek_char() {
            None => Err(LexError {
                span: Span::new(pos, pos),
                kind: LexErrorKind::UnexpectedEndOfInput,
            }),
            Some(ch) => Err(LexError {
                span: Span::new(pos, pos + ch.len_utf8()),
                kind: LexErrorKind::UnexpectedCharacter(ch),
            }),
        }
    }

    pub fn is_punct_next(&self) -> bool {
        Punct::is_punct(&self.inner)
    }

    pub fn is_ident_next(&self) -> bool {
        Ident::is_ident(&self.inner)
    }

    pub fn is_literal_next(&self) -> bool {
        Literal::is_literal(&self.inner)
    }

    pub fn is_delim_next(&self, delim: Delimiter) -> bool {
        if !Group::is_group(&self.inner) {
            return false;
        }
        match self.inner.peek_char() {
            Some('(') if delim == Delimiter::Parenthesis => true,
            Some('{') if delim == Delimiter::Brace => true,
            Some('[') if delim == Delimiter::Bracket => true,
            _ => false,
        }
    }
}

pub struct TokenStreamIterator {
    stream: TokenStream,
}

impl Iterator for TokenStreamIterator {
    type Item = Result<Token, LexError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.stream.is_empty() {
            None
        } else {
            Some(self.stream.next())
        }
    }
}

impl IntoIterator for TokenStream {
    type Item = Result<Token, LexError>;
    type IntoIter = TokenStreamIterator;

    #[inline]
    fn into_iter(self) -> TokenStreamIterator {
        TokenStreamIterator { stream: self }
    }
}
