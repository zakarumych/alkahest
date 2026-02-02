use core::{fmt, ops::Deref, str::FromStr};

use alloc::{rc::Rc, string::String, vec::Vec};

use crate::{
    Span,
    buffer::Buffer,
    lex::{Delimiter, LexError, LexErrorKind, Spacing, Token, TokenStream},
    parse::ParseStream,
};

trait Parser {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError>
    where
        Self: Sized;
}

trait Peek {
    fn peek(stream: &TokenStream) -> bool
    where
        Self: Sized;
}

impl Peek for Ident {
    #[inline]
    fn peek(stream: &TokenStream) -> bool {
        stream.is_ident_next()
    }
}

// ()
struct Parenthesis;

impl Peek for Parenthesis {
    #[inline]
    fn peek(stream: &TokenStream) -> bool {
        stream.is_delim_next(Delimiter::Parenthesis)
    }
}

// {}
struct Brace;

impl Peek for Brace {
    #[inline]
    fn peek(stream: &TokenStream) -> bool {
        stream.is_delim_next(Delimiter::Brace)
    }
}

// []
struct Bracket;

impl Peek for Bracket {
    #[inline]
    fn peek(stream: &TokenStream) -> bool {
        stream.is_delim_next(Delimiter::Bracket)
    }
}

impl TokenStream {
    fn parse<T: Parser>(&mut self) -> Result<T, ParseError> {
        T::parse(self)
    }

    fn peek<T: Peek>(&self) -> bool {
        T::peek(self)
    }
}

macro_rules! Token {
    (.) => {
        Dot
    };
    (,) => {
        Comma
    };
    (;) => {
        Semicolon
    };
    (:) => {
        Colon
    };
    (::) => {
        DoubleColon
    };
    (=) => {
        Equal
    };
    (&) => {
        Ampersand
    };
    (|) => {
        Pipe
    };
}

macro_rules! single_punct_token {
    ($p:tt as $name:ident) => {
        pub struct $name {
            _private: (),
        }

        impl Parser for $name {
            fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
                match stream.next()? {
                    Token::Punct(punct) if punct.eq_str(stringify!($p)) => {
                        Ok($name { _private: () })
                    }
                    token => Err(ParseError {
                        span: token.span(),
                        kind: ParseErrorKind::UnexpectedToken,
                    }),
                }
            }
        }

        impl Peek for $name {
            fn peek(stream: &TokenStream) -> bool {
                if !stream.is_punct_next() {
                    return false;
                }
                stream.fork().parse::<Self>().is_ok()
            }
        }
    };
}

macro_rules! double_punct_token {
    ($a:tt $b:tt as $name:ident) => {
        pub struct $name {
            _private: (),
        }

        impl Parser for $name {
            fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
                match stream.next()? {
                    Token::Punct(punct_a)
                        if punct_a.eq_str(stringify!($a))
                            && punct_a.spacing() == Spacing::Joint =>
                    {
                        match stream.next()? {
                            Token::Punct(punct_b) if punct_b.eq_str(stringify!($b)) => {
                                Ok($name { _private: () })
                            }
                            token => Err(ParseError {
                                span: token.span(),
                                kind: ParseErrorKind::UnexpectedToken,
                            }),
                        }
                    }
                    token => Err(ParseError {
                        span: token.span(),
                        kind: ParseErrorKind::UnexpectedToken,
                    }),
                }
            }
        }

        impl Peek for $name {
            fn peek(stream: &TokenStream) -> bool {
                if !stream.is_punct_next() {
                    return false;
                }
                stream.fork().parse::<Self>().is_ok()
            }
        }
    };
}

single_punct_token!(. as Dot);
single_punct_token!(, as Comma);
single_punct_token!(; as Semicolon);
single_punct_token!(: as Colon);
single_punct_token!(= as Equal);
single_punct_token!(& as Ampersand);
single_punct_token!(| as Pipe);
double_punct_token!(: : as DoubleColon);

macro_rules! keyword_token {
    ($name:ident) => {
        #[allow(non_camel_case_types)]
        pub struct $name {
            _private: (),
        }

        impl Parser for $name {
            fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
                match stream.next()? {
                    Token::Ident(ident) if ident.as_str() == stringify!($name) => {
                        Ok($name { _private: () })
                    }
                    token => Err(ParseError {
                        span: token.span(),
                        kind: ParseErrorKind::UnexpectedToken,
                    }),
                }
            }
        }

        impl Peek for $name {
            fn peek(stream: &TokenStream) -> bool {
                if !stream.is_ident_next() {
                    return false;
                }
                stream.fork().parse::<Self>().is_ok()
            }
        }
    };
}

keyword_token!(formula);
keyword_token!(pour);

macro_rules! parse_group {
    (@ $delimiter:ident $stream:ident $closure:expr) => {
        match $stream.next()? {
            Token::Group(group) if group.delimiter() == Delimiter::$delimiter => {
                let mut group_stream = group.stream();
                let parsed = $closure(&mut group_stream)?;
                if !group_stream.is_empty() {
                    return Err(ParseError {
                        span: group_stream.span(),
                        kind: ParseErrorKind::Unexpected,
                    });
                }
                parsed
            }
            Token::Group(group) => {
                return Err(ParseError {
                    span: group.span(),
                    kind: ParseErrorKind::UnexpectedToken,
                })
            }
            token => {
                return Err(ParseError {
                    span: token.span(),
                    kind: ParseErrorKind::UnexpectedToken,
                })
            }
        }
    };
}

macro_rules! parse_parenthesised {
    ($group:ident in $stream:ident) => {
        parse_group!(@ Parenthesis $stream <$group as Parser>::parse)
    };

    ($stream:ident => $body:expr) => {
        parse_group!(@ Parenthesis $stream |$stream: &mut TokenStream| -> Result<_, ParseError> { $body })
    };
}

macro_rules! parse_bracketed {
    ($group:ident in $stream:ident) => {
        parse_group!(@ Bracket $stream <$group as Parser>::parse)
    };

    ($stream:ident => $body:expr) => {
        parse_group!(@ Bracket $stream |$stream: &mut TokenStream| -> Result<_, ParseError> { $body })
    };
}

macro_rules! parse_braced {
    ($group:ident in $stream:ident) => {
        parse_group!(@ Brace $stream <$group as Parser>::parse)
    };

    ($stream:ident => $body:expr) => {
        parse_group!(@ Brace $stream |$stream: &mut TokenStream| -> Result<_, ParseError> { $body })
    };
}

macro_rules! parse_terminated {
    (by $delim:ident in $stream:ident => $body:expr) => {{
        struct ParseTerminated<'a, F> {
            stream: &'a mut TokenStream,
            f: F,
            exhausted: bool,
        }

        impl<'a, F, T> Iterator for ParseTerminated<'a, F>
        where
            F: FnMut(&mut TokenStream) -> Result<T, ParseError>,
        {
            type Item = Result<T, ParseError>;

            fn next(&mut self) -> Option<Self::Item> {
                if self.exhausted || self.stream.is_empty() {
                    return None;
                }

                let element = (self.f)(self.stream);

                if self.stream.peek::<$delim>() {
                    match self.stream.parse::<$delim>() {
                        Ok(_) => Some(element),
                        Err(err) => Some(Err(err))
                    }
                } else {
                    self.exhausted = true;
                    Some(element)
                }
            }
        }

        ParseTerminated { stream: $stream, f: |$stream: &mut TokenStream| -> Result<_, ParseError> { $body }, exhausted: false }
    }};

    ($element:ident by $delim:ident in $stream:ident) => {{
        parse_terminated!(by $delim in $stream => $element::parse($stream))
    }};
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Ident(Rc<str>);

impl Ident {
    pub fn new(name: &str) -> Self {
        Ident(Rc::from(name))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl Deref for Ident {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.0.as_ref()
    }
}

impl AsRef<str> for Ident {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Parser for Ident {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
        match stream.next()? {
            Token::Ident(ident) => Ok(Ident(ident.clone_rc_str())),
            token => Err(ParseError {
                span: token.span(),
                kind: ParseErrorKind::UnexpectedToken,
            }),
        }
    }
}

impl Parser for u32 {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
        match stream.next()? {
            Token::Literal(lit) if lit.as_str().starts_with(|ch: char| ch.is_ascii_digit()) => {
                match u32::from_str(lit.as_str()) {
                    Ok(value) => Ok(value),
                    Err(_) => Err(ParseError {
                        span: lit.span(),
                        kind: ParseErrorKind::UnexpectedToken,
                    }),
                }
            }
            token => Err(ParseError {
                span: token.span(),
                kind: ParseErrorKind::UnexpectedToken,
            }),
        }
    }
}

/// A reference to a symbol (e.g., formula name).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path(Rc<str>);

struct SkipLastIterator<I>
where
    I: Iterator,
{
    iter: I,
    last: Option<I::Item>,
}

impl<I> Iterator for SkipLastIterator<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => None,
            Some(item) => match self.last.take() {
                None => {
                    self.last = Some(item);
                    self.next()
                }
                Some(last) => {
                    self.last = Some(item);
                    Some(last)
                }
            },
        }
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let mut acc = init;
        let mut last = self.last;

        for item in self.iter {
            match last {
                None => last = Some(item),
                Some(l) => {
                    last = Some(item);
                    acc = f(acc, l);
                }
            }
        }

        acc
    }
}

impl Path {
    pub fn path(&self) -> impl Iterator<Item = &str> + '_ {
        SkipLastIterator {
            iter: self.0.split('.'),
            last: None,
        }
    }

    pub fn name(&self) -> &str {
        match self.0.rsplit_once('.') {
            Some((_, name)) => name,
            None => &self.0,
        }
    }
}

impl Parser for Path {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
        let mut full_name = String::new();

        if stream.peek::<Token![.]>() {
            // Leading dot indicates global path.
            let _dot = stream.parse::<Token![.]>()?;
            full_name.push('.');
        }

        let first_ident = stream.parse::<Ident>()?;
        full_name.push_str(first_ident.as_str());

        while stream.peek::<Token![.]>() {
            let _dot = stream.parse::<Token![.]>()?;
            let next_ident = stream.parse::<Ident>()?;
            full_name.push('.');
            full_name.push_str(&next_ident.as_str());
        }

        Ok(Path(Rc::from(full_name)))
    }
}

/// Formula of an element.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Element {
    pub indirect: bool,
    pub kind: ElementKind,
}

impl Parser for Element {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError>
    where
        Self: Sized,
    {
        let mut indirect = false;
        if stream.peek::<Token![&]>() {
            let _amp = stream.parse::<Token![&]>()?;
            indirect = true;
        }

        let kind = stream.parse::<ElementKind>()?;

        Ok(Element { indirect, kind })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct List {
    pub element: Rc<Element>,
    pub min_len: u32,
    pub max_len: u32,
}

impl Parser for List {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
        let element = stream.parse::<Element>()?;

        if !stream.peek::<Token![;]>() {
            return Ok(List {
                element: Rc::new(element),
                min_len: 0,
                max_len: u32::MAX,
            });
        }

        let _semicolon = stream.parse::<Token![;]>()?;
        let size = stream.parse::<u32>()?;

        if !stream.peek::<Token![:]>() {
            return Ok(List {
                element: Rc::new(element),
                min_len: size,
                max_len: size,
            });
        }

        let _colon = stream.parse::<Token![:]>()?;
        let max_size = stream.parse::<u32>()?;

        Ok(List {
            element: Rc::new(element),
            min_len: size,
            max_len: max_size,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tuple {
    pub elements: Rc<[Element]>,
}

impl Parser for Tuple {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
        let elements = parse_terminated!(Element by Comma in stream);

        Ok(Tuple {
            elements: elements.collect::<Result<_, ParseError>>()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElementKind {
    /// A reference to another formula.
    Symbol(Path),

    /// A sequence of elements with the same formula.
    ///
    /// Length bounds are inclusive.
    ///
    /// Bounds affect how length is serialized.
    List(List),

    /// A fixed-length sequence of elements with different formulas.
    Tuple(Tuple),
}

impl Parser for ElementKind {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError>
    where
        Self: Sized,
    {
        if stream.peek::<Ident>() {
            let symbol = stream.parse::<Path>()?;
            return Ok(ElementKind::Symbol(symbol));
        }

        if stream.peek::<Bracket>() {
            let list = parse_bracketed!(List in stream);

            return Ok(ElementKind::List(list));
        }

        if stream.peek::<Parenthesis>() {
            let tuple = parse_parenthesised!(Tuple in stream);
            return Ok(ElementKind::Tuple(tuple));
        }

        Err(ParseError {
            span: stream.span(),
            kind: ParseErrorKind::ExpectedFormulaRef,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedElement {
    pub name: Ident,
    pub element: Element,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Record {
    pub fields: Rc<[NamedElement]>,
}

impl Parser for Record {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
        let fields = parse_terminated!(by Comma in stream => {
            let field_name = stream.parse::<Ident>()?;
            let _colon = stream.parse::<Token![:]>()?;
            let field_element = stream.parse::<Element>()?;

            Ok(NamedElement {
                name: field_name,
                element: field_element,
            })
        });

        Ok(Record {
            fields: fields.collect::<Result<_, ParseError>>()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Variant {
    Unit,

    /// A fixed-length sequence of elements with different formulas.
    Tuple(Tuple),

    /// A fixed-length structure with named fields.
    Record(Record),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedVariant {
    pub name: Ident,
    pub variant: Variant,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Variants(pub Rc<[NamedVariant]>);

impl Parser for Variants {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
        let mut variants = Vec::new();

        while stream.peek::<Token![|]>() {
            let _pipe = stream.parse::<Token![|]>()?;
            let variant_name = stream.parse::<Ident>()?;

            if stream.peek::<Parenthesis>() {
                let tuple_formula = parse_parenthesised!(Tuple in stream);

                variants.push(NamedVariant {
                    name: variant_name,
                    variant: Variant::Tuple(tuple_formula),
                });
            } else if stream.peek::<Brace>() {
                let record_formula = parse_braced!(Record in stream);

                variants.push(NamedVariant {
                    name: variant_name,
                    variant: Variant::Record(record_formula),
                });
            } else {
                variants.push(NamedVariant {
                    name: variant_name,
                    variant: Variant::Unit,
                });
            }
        }

        Ok(Variants(Rc::from(variants)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Formula {
    /// An empty formula.
    Unit,

    /// A fixed-length sequence of elements with different formulas.
    Tuple(Tuple),

    /// A fixed-length structure with named fields.
    Record(Record),

    /// An alternative among different variants.
    Variants(Variants),
}

impl Parser for Formula {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
        if stream.peek::<Parenthesis>() {
            let tuple_formula = parse_parenthesised!(Tuple in stream);
            return Ok(Formula::Tuple(tuple_formula));
        }

        if stream.peek::<Brace>() {
            let record_formula = parse_braced!(Record in stream);
            return Ok(Formula::Record(record_formula));
        }

        if stream.peek::<Token![|]>() {
            let variants_formula = stream.parse::<Variants>()?;
            return Ok(Formula::Variants(variants_formula));
        }

        Err(ParseError {
            span: stream.span(),
            kind: ParseErrorKind::ExpectedFormula,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Definition {
    pub name: Ident,
    pub generics: Rc<[Ident]>,
    pub formula: Formula,
}

impl Peek for Definition {
    fn peek(stream: &TokenStream) -> bool {
        stream.peek::<formula>()
    }
}

impl Parser for Definition {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
        formula::parse(stream)?;
        let name = stream.parse::<Ident>()?;
        let mut generics = Vec::new();

        while stream.peek::<Ident>() {
            let generic = stream.parse::<Ident>()?;
            generics.push(generic);
        }

        if stream.peek::<Token![;]>() {
            let _semicolon = stream.parse::<Token![;]>()?;

            return Ok(Definition {
                name,
                generics: Rc::from(generics),
                formula: Formula::Unit,
            });
        }

        let _eq = stream.parse::<Token![=]>()?;
        let formula = stream.parse::<Formula>()?;
        let _semicolon = stream.parse::<Token![;]>()?;

        Ok(Definition {
            name,
            generics: Rc::from(generics),
            formula,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImportTree {
    pub path: Path,
    pub branches: Option<Rc<[ImportTree]>>,
}

impl Parser for ImportTree {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
        let first_ident = stream.parse::<Ident>()?;

        let mut full_path = String::new();
        full_path.push_str(first_ident.as_str());

        while stream.peek::<Token![.]>() {
            let _dot = stream.parse::<Token![.]>()?;

            if stream.peek::<Brace>() {
                let branches = parse_braced!(stream => {
                    Ok(parse_terminated!(ImportTree by Comma in stream).collect::<Result<_, ParseError>>()?)
                });
                return Ok(ImportTree {
                    path: Path(Rc::from(full_path)),
                    branches: Some(branches),
                });
            }

            let next_ident = stream.parse::<Ident>()?;
            full_path.push('.');
            full_path.push_str(&next_ident.as_str());
        }

        Ok(ImportTree {
            path: Path(Rc::from(full_path)),
            branches: None,
        })
    }
}

/// Represents entire source file containing Alkahest formulas.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Module {
    pub imports: Rc<[ImportTree]>,

    /// List of formulas defined in the document.
    pub definitions: Rc<[Definition]>,
}

impl Parser for Module {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
        let mut imports = Vec::new();
        let mut definitions = Vec::new();

        while !stream.is_empty() {
            if stream.peek::<formula>() {
                let definition = stream.parse::<Definition>()?;
                definitions.push(definition);
            } else if stream.peek::<pour>() {
                let _pour = stream.parse::<pour>()?;

                if stream.peek::<Brace>() {
                    parse_braced!(stream => {
                        for tree in parse_terminated!(ImportTree by Comma in stream) {
                            imports.push(tree?);
                        }
                        Ok(())
                    });
                } else {
                    let import_tree = stream.parse::<ImportTree>()?;
                    imports.push(import_tree);
                }

                stream.parse::<Token![;]>()?;
            } else {
                return Err(ParseError {
                    span: stream.span(),
                    kind: ParseErrorKind::Unexpected,
                });
            }
        }

        Ok(Module {
            imports: Rc::from(imports),
            definitions: Rc::from(definitions),
        })
    }
}

#[derive(Clone, Debug)]
pub enum ParseErrorKind {
    LexError(LexErrorKind),
    Unexpected,
    UnexpectedToken,
    ExpectedFormula,
    ExpectedFormulaRef,
    Custom(String),
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErrorKind::LexError(kind) => write!(f, "{:?}", kind),
            ParseErrorKind::Unexpected => write!(f, "Unexpected token"),
            ParseErrorKind::UnexpectedToken => write!(f, "Unexpected token"),
            ParseErrorKind::ExpectedFormula => write!(f, "Expected formula"),
            ParseErrorKind::ExpectedFormulaRef => write!(f, "Expected formula reference"),
            ParseErrorKind::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ParseError {
    span: Span,
    kind: ParseErrorKind,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}..{}", self.kind, self.span.start, self.span.end)
    }
}

impl ParseError {
    pub(crate) fn new(span: Span, kind: ParseErrorKind) -> Self {
        ParseError { span, kind }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn kind(&self) -> &ParseErrorKind {
        &self.kind
    }
}

impl From<LexError> for ParseError {
    fn from(err: LexError) -> Self {
        ParseError {
            span: err.span(),
            kind: ParseErrorKind::LexError(err.kind()),
        }
    }
}

pub fn parse_module(source: String) -> Result<Module, ParseError> {
    let mut token_stream = TokenStream::new(ParseStream::new(Buffer::from_string(source)));
    let module = token_stream.parse::<Module>()?;

    if !token_stream.is_empty() {
        return Err(ParseError {
            span: token_stream.span(),
            kind: ParseErrorKind::Unexpected,
        });
    }

    Ok(module)
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    fn parse_formula(source: &str) -> Result<Formula, ParseError> {
        let mut stream =
            TokenStream::new(ParseStream::new(Buffer::from_string(source.to_string())));
        stream.parse::<Formula>()
    }

    fn parse_element(source: &str) -> Result<Element, ParseError> {
        let mut stream =
            TokenStream::new(ParseStream::new(Buffer::from_string(source.to_string())));
        stream.parse::<Element>()
    }

    fn parse_module(source: &str) -> Result<Module, ParseError> {
        let mut stream =
            TokenStream::new(ParseStream::new(Buffer::from_string(source.to_string())));
        stream.parse::<Module>()
    }

    #[test]
    fn test_symbol_simple() {
        let element = parse_element("MyType").unwrap();
        match element.kind {
            ElementKind::Symbol(sym) => assert_eq!(sym.name(), "MyType"),
            _ => panic!("Expected Symbol"),
        }
    }

    #[test]
    fn test_symbol_with_path() {
        let element = parse_element("module::Type").unwrap();
        match element.kind {
            ElementKind::Symbol(sym) => {
                assert_eq!(sym.name(), "Type");
                assert_eq!(sym.path().collect::<Vec<_>>(), alloc::vec!["module"]);
            }
            _ => panic!("Expected Symbol"),
        }
    }

    #[test]
    fn test_symbol_nested_path() {
        let element = parse_element("a::b::c::Type").unwrap();
        match element.kind {
            ElementKind::Symbol(sym) => {
                assert_eq!(sym.name(), "Type");
                assert_eq!(sym.path().collect::<Vec<_>>(), alloc::vec!["a", "b", "c"]);
            }
            _ => panic!("Expected Symbol"),
        }
    }

    #[test]
    fn test_reference() {
        let element = parse_element("&MyType").unwrap();

        assert!(element.indirect);

        match element.kind {
            ElementKind::Symbol(sym) => assert_eq!(sym.name(), "MyType"),
            _ => panic!("Expected Symbol inside Reference"),
        }
    }

    #[test]
    fn test_list_unbounded() {
        let element = parse_element("[u32]").unwrap();
        match element.kind {
            ElementKind::List(List {
                element,
                min_len,
                max_len,
            }) => {
                assert_eq!(min_len, 0);
                assert_eq!(max_len, u32::MAX);
                assert_eq!(element.indirect, false);
                match &element.kind {
                    ElementKind::Symbol(sym) => assert_eq!(sym.name(), "u32"),
                    _ => panic!("Expected Symbol"),
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_list_fixed_length() {
        let element = parse_element("[u32; 10]").unwrap();
        match element.kind {
            ElementKind::List(List {
                element,
                min_len,
                max_len,
            }) => {
                assert_eq!(min_len, 10);
                assert_eq!(max_len, 10);
                assert_eq!(element.indirect, false);
                match &element.kind {
                    ElementKind::Symbol(sym) => assert_eq!(sym.name(), "u32"),
                    _ => panic!("Expected Symbol"),
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_list_bounded() {
        let element = parse_element("[u32; 5:10]").unwrap();
        match element.kind {
            ElementKind::List(List {
                element,
                min_len,
                max_len,
            }) => {
                assert_eq!(min_len, 5);
                assert_eq!(max_len, 10);
                assert_eq!(element.indirect, false);
                match &element.kind {
                    ElementKind::Symbol(sym) => assert_eq!(sym.name(), "u32"),
                    _ => panic!("Expected Symbol"),
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_list_of_references() {
        let element = parse_element("[&Data; 1:100]").unwrap();
        match element.kind {
            ElementKind::List(List {
                element,
                min_len,
                max_len,
            }) => {
                assert_eq!(min_len, 1);
                assert_eq!(max_len, 100);
                assert_eq!(element.indirect, true);
                match &element.kind {
                    ElementKind::Symbol(sym) => assert_eq!(sym.name(), "Data"),
                    _ => panic!("Expected Symbol"),
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_deeply_nested() {
        let element = parse_element("[[u32]]").unwrap();
        match element.kind {
            ElementKind::List(List { element, .. }) => match &element.kind {
                ElementKind::List(List { element: inner, .. }) => match &inner.kind {
                    ElementKind::Symbol(sym) => assert_eq!(sym.name(), "u32"),
                    _ => panic!("Expected Symbol"),
                },
                _ => panic!("Expected List"),
            },
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_tuple_empty() {
        let element = parse_element("()").unwrap();
        match element.kind {
            ElementKind::Tuple(Tuple { elements }) => {
                assert_eq!(elements.len(), 0);
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_tuple_single_element() {
        let element = parse_element("(u32)").unwrap();
        match element.kind {
            ElementKind::Tuple(Tuple { elements }) => {
                assert_eq!(elements.len(), 1);
                assert_eq!(elements[0].indirect, false);
                match &elements[0].kind {
                    ElementKind::Symbol(sym) => assert_eq!(sym.name(), "u32"),
                    _ => panic!("Expected Symbol"),
                }
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_tuple_multiple_elements() {
        let element = parse_element("(u32, &string, bool)").unwrap();
        match element.kind {
            ElementKind::Tuple(Tuple { elements }) => {
                assert_eq!(elements.len(), 3);
                assert_eq!(elements[0].indirect, false);
                assert_eq!(elements[1].indirect, true);
                assert_eq!(elements[2].indirect, false);
                match &elements[0].kind {
                    ElementKind::Symbol(sym) => assert_eq!(sym.name(), "u32"),
                    _ => panic!("Expected Symbol"),
                }
                match &elements[1].kind {
                    ElementKind::Symbol(sym) => assert_eq!(sym.name(), "string"),
                    _ => panic!("Expected Symbol"),
                }
                match &elements[2].kind {
                    ElementKind::Symbol(sym) => assert_eq!(sym.name(), "bool"),
                    _ => panic!("Expected Symbol"),
                }
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_tuple_with_complex_types() {
        let element = parse_element("([u32; 5], &Data)").unwrap();
        match element.kind {
            ElementKind::Tuple(Tuple { elements }) => {
                assert_eq!(elements.len(), 2);

                assert_eq!(elements[0].indirect, false);
                match &elements[0].kind {
                    ElementKind::List(List {
                        min_len, max_len, ..
                    }) => {
                        assert_eq!(*min_len, 5);
                        assert_eq!(*max_len, 5);
                    }
                    _ => panic!("Expected List"),
                }

                assert_eq!(elements[1].indirect, true);
                match &elements[1].kind {
                    ElementKind::Symbol(sym) => assert_eq!(sym.name(), "Data"),
                    _ => panic!("Expected Reference"),
                }
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_record_single_field() {
        let formula = parse_formula("{x: u32}").unwrap();
        match formula {
            Formula::Record(Record { fields }) => {
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].name.as_str(), "x");
                assert_eq!(fields[0].element.indirect, false);
                match &fields[0].element.kind {
                    ElementKind::Symbol(sym) => assert_eq!(sym.name(), "u32"),
                    _ => panic!("Expected Symbol"),
                }
            }
            _ => panic!("Expected Record"),
        }
    }

    #[test]
    fn test_record_multiple_fields() {
        let formula = parse_formula("{x: u32, y: string, z: bool}").unwrap();
        match formula {
            Formula::Record(Record { fields }) => {
                assert_eq!(fields.len(), 3);
                assert_eq!(fields[0].name.as_str(), "x");
                assert_eq!(fields[1].name.as_str(), "y");
                assert_eq!(fields[2].name.as_str(), "z");
            }
            _ => panic!("Expected Record"),
        }
    }

    #[test]
    fn test_record_with_complex_types() {
        let formula = parse_formula("{data: [u32; 10], ptr: &Data}").unwrap();
        match formula {
            Formula::Record(Record { fields }) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name.as_str(), "data");
                match &fields[0].element.kind {
                    ElementKind::List(List {
                        min_len, max_len, ..
                    }) => {
                        assert_eq!(*min_len, 10);
                        assert_eq!(*max_len, 10);
                    }
                    _ => panic!("Expected List"),
                }
                assert_eq!(fields[1].name.as_str(), "ptr");
                assert_eq!(fields[1].element.indirect, true);
                match &fields[1].element.kind {
                    ElementKind::Symbol(sym) => assert_eq!(sym.name(), "Data"),
                    _ => panic!("Expected Symbol"),
                }
            }
            _ => panic!("Expected Record"),
        }
    }

    #[test]
    fn test_variant_single() {
        let formula = parse_formula("|Foo(u32)").unwrap();
        match formula {
            Formula::Variants(Variants(variants)) => {
                assert_eq!(variants.len(), 1);
                assert_eq!(variants[0].name.as_str(), "Foo");
                match &variants[0].variant {
                    Variant::Tuple(Tuple { elements }) => {
                        assert_eq!(elements.len(), 1);
                        assert_eq!(elements[0].indirect, false);
                        match &elements[0].kind {
                            ElementKind::Symbol(sym) => assert_eq!(sym.name(), "u32"),
                            _ => panic!("Expected Symbol"),
                        }
                    }
                    _ => panic!("Expected Tuple variant"),
                }
            }
            _ => panic!("Expected Variant"),
        }
    }

    #[test]
    fn test_variant_multiple() {
        let formula = parse_formula("|None |Some(u32)").unwrap();
        match formula {
            Formula::Variants(Variants(variants)) => {
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].name.as_str(), "None");
                assert_eq!(variants[1].name.as_str(), "Some");
            }
            _ => panic!("Expected Variant"),
        }
    }

    #[test]
    fn test_variant_with_complex_types() {
        let formula = parse_formula("|Ok {value: u32} |Err(&string)").unwrap();
        match formula {
            Formula::Variants(Variants(variants)) => {
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].name.as_str(), "Ok");
                match &variants[0].variant {
                    Variant::Record(Record { fields }) => {
                        assert_eq!(fields.len(), 1);
                        assert_eq!(fields[0].name.as_str(), "value");
                        assert_eq!(fields[0].element.indirect, false);
                        match &fields[0].element.kind {
                            ElementKind::Symbol(sym) => assert_eq!(sym.name(), "u32"),
                            _ => panic!("Expected Symbol"),
                        }
                    }
                    _ => panic!("Expected Record"),
                }
                assert_eq!(variants[1].name.as_str(), "Err");
                match &variants[1].variant {
                    Variant::Tuple(Tuple { elements }) => {
                        assert_eq!(elements.len(), 1);
                        assert_eq!(elements[0].indirect, true);
                        match &elements[0].kind {
                            ElementKind::Symbol(sym) => assert_eq!(sym.name(), "string"),
                            _ => panic!("Expected Symbol"),
                        }
                    }
                    _ => panic!("Expected Tuple variant"),
                }
            }
            _ => panic!("Expected Variants"),
        }
    }

    #[test]
    fn test_module_simple() {
        let module = parse_module("formula MyType = (u32);").unwrap();
        assert_eq!(module.definitions.len(), 1);
        assert_eq!(module.definitions[0].name.as_str(), "MyType");
        match &module.definitions[0].formula {
            Formula::Tuple(Tuple { elements }) => {
                assert_eq!(elements.len(), 1);
                assert_eq!(elements[0].indirect, false);
                match &elements[0].kind {
                    ElementKind::Symbol(sym) => assert_eq!(sym.name(), "u32"),
                    _ => panic!("Expected Symbol"),
                }
            }
            _ => panic!("Expected Symbol"),
        }
    }

    #[test]
    fn test_module_multiple_formulas() {
        let source = "formula A = (u32); formula B = (string); formula C = {x: u32}; formula D = |Foo |Bar(u32) |Baz {a: A, b: B, c: C};";
        let module = parse_module(source).unwrap();
        assert_eq!(module.definitions.len(), 4);
        assert_eq!(module.definitions[0].name.as_str(), "A");
        assert_eq!(module.definitions[1].name.as_str(), "B");
        assert_eq!(module.definitions[2].name.as_str(), "C");
        assert_eq!(module.definitions[3].name.as_str(), "D");
    }

    #[test]
    fn test_multiline() {
        let source = r#"    
formula Foo = {
    a: u32,
    b: u32,
};
"#;
        let module = parse_module(source).unwrap();
        assert_eq!(module.definitions.len(), 1);
        assert_eq!(module.definitions[0].name.as_str(), "Foo");
    }
}
