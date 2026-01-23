use std::{fmt, ops::Deref, rc::Rc, str::FromStr};

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

struct Parenthesis;

impl Peek for Parenthesis {
    #[inline]
    fn peek(stream: &TokenStream) -> bool {
        stream.is_delim_next(Delimiter::Parenthesis)
    }
}

struct Bracket;

impl Peek for Bracket {
    #[inline]
    fn peek(stream: &TokenStream) -> bool {
        stream.is_delim_next(Delimiter::Bracket)
    }
}

struct Brace;

impl Peek for Brace {
    #[inline]
    fn peek(stream: &TokenStream) -> bool {
        stream.is_delim_next(Delimiter::Brace)
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
                    Token::Punct(punct) if punct.eq_str(std::stringify!($p)) => {
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
                        if punct_a.eq_str(std::stringify!($a))
                            && punct_a.spacing() == Spacing::Joint =>
                    {
                        match stream.next()? {
                            Token::Punct(punct_b) if punct_b.eq_str(std::stringify!($b)) => {
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
                    Token::Ident(ident) if ident.as_str() == std::stringify!($name) => {
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

macro_rules! parse_group {
    (@ $delimiter:ident $group:ident $stream:ident) => {
        match $stream.next()? {
            Token::Group(group) if group.delimiter() == Delimiter::$delimiter => {
                let mut group_stream = group.stream();
                let parsed = <$group as Parser>::parse(&mut group_stream)?;
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
        parse_group!(@ Parenthesis $group $stream)
    };
}

macro_rules! parse_bracketed {
    ($group:ident in $stream:ident) => {
        parse_group!(@ Bracket $group $stream)
    };
}

macro_rules! parse_braced {
    ($group:ident in $stream:ident) => {
        parse_group!(@ Brace $group $stream)
    };
}

#[derive(Clone)]
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

    #[inline(always)]
    fn deref(&self) -> &str {
        self.0.as_ref()
    }
}

impl AsRef<str> for Ident {
    #[inline(always)]
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
#[derive(Clone)]
pub struct Symbol(Rc<str>);

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

impl Symbol {
    pub fn path(&self) -> impl Iterator<Item = &str> + '_ {
        SkipLastIterator {
            iter: self.0.split("::"),
            last: None,
        }
    }

    pub fn name(&self) -> &str {
        match self.0.rsplit_once("::") {
            Some((_, name)) => name,
            None => &self.0,
        }
    }
}

impl Parser for Symbol {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
        let mut full_name = String::new();

        let first_ident = stream.parse::<Ident>()?;
        full_name.push_str(first_ident.as_str());

        while stream.peek::<Token![::]>() {
            let _double_colon = stream.parse::<Token![::]>()?;
            let next_ident = stream.parse::<Ident>()?;
            full_name.push_str("::");
            full_name.push_str(&next_ident.as_str());
        }

        Ok(Symbol(Rc::from(full_name)))
    }
}

#[derive(Clone)]
pub struct NamedFormula {
    pub name: Ident,
    pub formula: Formula,
}

/// Data formula.
#[derive(Clone)]
pub enum Formula {
    /// A reference to another formula.
    Symbol(Symbol),

    /// A formula of reference.
    /// Encoded as a pointer to another location that holds the actual data.
    Reference { formula: Rc<Formula> },

    /// A sequence of elements with the same formula.
    ///
    /// Length bounds are inclusive.
    ///
    /// Bounds affect how length is serialized.
    List {
        element: Rc<Formula>,
        min_len: u32,
        max_len: u32,
    },

    /// A fixed-length sequence of elements with different formulas.
    Tuple { elements: Rc<[Formula]> },

    /// A fixed-length structure with named fields.
    Record { fields: Rc<[NamedFormula]> },

    /// An alternative among different variants.
    Variant { variants: Rc<[NamedFormula]> },
}

struct ListFormula {
    element: Formula,
    min_len: u32,
    max_len: u32,
}

impl Parser for ListFormula {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
        let element = stream.parse::<Formula>()?;

        if !stream.peek::<Token![;]>() {
            return Ok(ListFormula {
                element,
                min_len: 0,
                max_len: u32::MAX,
            });
        }

        let _semicolon = stream.parse::<Token![;]>()?;
        let size = stream.parse::<u32>()?;

        if !stream.peek::<Token![:]>() {
            return Ok(ListFormula {
                element,
                min_len: size,
                max_len: size,
            });
        }

        let _colon = stream.parse::<Token![:]>()?;
        let max_size = stream.parse::<u32>()?;

        Ok(ListFormula {
            element,
            min_len: size,
            max_len: max_size,
        })
    }
}

struct TupleFormula {
    elements: Vec<Formula>,
}

impl Parser for TupleFormula {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
        let mut elements = Vec::new();

        while !stream.is_empty() {
            let element = stream.parse::<Formula>()?;
            elements.push(element);

            if stream.peek::<Comma>() {
                let _comma = stream.parse::<Comma>()?;
            } else {
                break;
            }
        }

        Ok(TupleFormula { elements })
    }
}

struct RecordFormula {
    fields: Vec<NamedFormula>,
}

impl Parser for RecordFormula {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
        let mut fields = Vec::new();

        while !stream.is_empty() {
            let field_name = stream.parse::<Ident>()?;
            let _colon = stream.parse::<Token![:]>()?;
            let field_formula = stream.parse::<Formula>()?;

            fields.push(NamedFormula {
                name: field_name,
                formula: field_formula,
            });

            if stream.peek::<Comma>() {
                let _comma = stream.parse::<Comma>()?;
            } else {
                break;
            }
        }

        Ok(RecordFormula { fields })
    }
}

struct VariantFormula {
    variants: Vec<NamedFormula>,
}

impl Parser for VariantFormula {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
        let mut variants = Vec::new();

        while stream.peek::<Token![|]>() {
            let _pipe = stream.parse::<Token![|]>()?;
            let variant_name = stream.parse::<Ident>()?;

            // If variant name is followed by next variant, comma, or is at the end, it is a unit variant.
            // This makes it impossible to define variant which formula is an enum.
            if stream.peek::<Token![|]>() || stream.is_empty() || stream.peek::<Token![,]>() {
                // Variant without associated formula (unit variant).
                variants.push(NamedFormula {
                    name: variant_name,
                    formula: Formula::Tuple {
                        elements: Rc::from([]),
                    },
                });
                continue;
            }

            let variant_formula = stream.parse::<Formula>()?;

            variants.push(NamedFormula {
                name: variant_name,
                formula: variant_formula,
            });
        }

        Ok(VariantFormula { variants })
    }
}

impl Parser for Formula {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
        if stream.peek::<Ident>() {
            let symbol = stream.parse::<Symbol>()?;
            return Ok(Formula::Symbol(symbol));
        }

        if stream.peek::<Token![&]>() {
            let _amp = stream.parse::<Token![&]>()?;
            let formula = stream.parse::<Formula>()?;
            return Ok(Formula::Reference {
                formula: Rc::new(formula),
            });
        }

        if stream.peek::<Bracket>() {
            let list_formula = parse_bracketed!(ListFormula in stream);

            return Ok(Formula::List {
                element: Rc::new(list_formula.element),
                min_len: list_formula.min_len,
                max_len: list_formula.max_len,
            });
        }

        if stream.peek::<Parenthesis>() {
            let tuple_formula = parse_parenthesised!(TupleFormula in stream);
            return Ok(Formula::Tuple {
                elements: Rc::from(tuple_formula.elements),
            });
        }

        if stream.peek::<Brace>() {
            let record_formula = parse_braced!(RecordFormula in stream);
            return Ok(Formula::Record {
                fields: Rc::from(record_formula.fields),
            });
        }

        if stream.peek::<Token![|]>() {
            let variant_formula = stream.parse::<VariantFormula>()?;
            return Ok(Formula::Variant {
                variants: Rc::from(variant_formula.variants),
            });
        }

        Err(ParseError {
            span: stream.span(),
            kind: ParseErrorKind::ExpectedFormula,
        })
    }
}

/// Represents entire source file containing Alkahest formulas.
pub struct Module {
    /// List of formulas defined in the document.
    pub formulas: Vec<NamedFormula>,
}

impl Parser for Module {
    fn parse(stream: &mut TokenStream) -> Result<Self, ParseError> {
        let mut formulas = Vec::new();

        while !stream.is_empty() {
            formula::parse(stream)?;
            let name = stream.parse::<Ident>()?;
            let _eq = stream.parse::<Token![=]>()?;
            let formula = stream.parse::<Formula>()?;
            let _semicolon = stream.parse::<Token![;]>()?;

            formulas.push(NamedFormula { name, formula });
        }

        Ok(Module { formulas })
    }
}

#[derive(Clone, Debug)]
pub enum ParseErrorKind {
    LexError(LexErrorKind),
    Unexpected,
    UnexpectedToken,
    ExpectedFormula,
    Custom(String),
}

#[derive(Clone, Debug)]
pub struct ParseError {
    span: Span,
    kind: ParseErrorKind,
}

impl ParseError {
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

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErrorKind::LexError(kind) => write!(f, "{:?}", kind),
            ParseErrorKind::Unexpected => write!(f, "Unexpected token"),
            ParseErrorKind::UnexpectedToken => write!(f, "Unexpected token"),
            ParseErrorKind::ExpectedFormula => write!(f, "Expected formula"),
            ParseErrorKind::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

pub fn parse_string(source: String) -> Result<Module, ParseError> {
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
    use super::*;

    fn parse_formula(source: &str) -> Result<Formula, ParseError> {
        let mut stream =
            TokenStream::new(ParseStream::new(Buffer::from_string(source.to_string())));
        stream.parse::<Formula>()
    }

    fn parse_module(source: &str) -> Result<Module, ParseError> {
        parse_string(source.to_string())
    }

    #[test]
    fn test_symbol_simple() {
        let formula = parse_formula("MyType").unwrap();
        match formula {
            Formula::Symbol(sym) => assert_eq!(sym.name(), "MyType"),
            _ => panic!("Expected Symbol"),
        }
    }

    #[test]
    fn test_symbol_with_path() {
        let formula = parse_formula("module::Type").unwrap();
        match formula {
            Formula::Symbol(sym) => {
                assert_eq!(sym.name(), "Type");
                assert_eq!(sym.path().collect::<Vec<_>>(), vec!["module"]);
            }
            _ => panic!("Expected Symbol"),
        }
    }

    #[test]
    fn test_symbol_nested_path() {
        let formula = parse_formula("a::b::c::Type").unwrap();
        match formula {
            Formula::Symbol(sym) => {
                assert_eq!(sym.name(), "Type");
                assert_eq!(sym.path().collect::<Vec<_>>(), vec!["a", "b", "c"]);
            }
            _ => panic!("Expected Symbol"),
        }
    }

    #[test]
    fn test_reference() {
        let formula = parse_formula("&MyType").unwrap();
        match formula {
            Formula::Reference { formula: inner } => match inner.as_ref() {
                Formula::Symbol(sym) => assert_eq!(sym.name(), "MyType"),
                _ => panic!("Expected Symbol inside Reference"),
            },
            _ => panic!("Expected Reference"),
        }
    }

    #[test]
    fn test_list_unbounded() {
        let formula = parse_formula("[u32]").unwrap();
        match formula {
            Formula::List {
                element,
                min_len,
                max_len,
            } => {
                assert_eq!(min_len, 0);
                assert_eq!(max_len, u32::MAX);
                match element.as_ref() {
                    Formula::Symbol(sym) => assert_eq!(sym.name(), "u32"),
                    _ => panic!("Expected Symbol"),
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_list_fixed_length() {
        let formula = parse_formula("[u32; 10]").unwrap();
        match formula {
            Formula::List {
                element,
                min_len,
                max_len,
            } => {
                assert_eq!(min_len, 10);
                assert_eq!(max_len, 10);
                match element.as_ref() {
                    Formula::Symbol(sym) => assert_eq!(sym.name(), "u32"),
                    _ => panic!("Expected Symbol"),
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_list_bounded() {
        let formula = parse_formula("[u32; 5:10]").unwrap();
        match formula {
            Formula::List {
                element,
                min_len,
                max_len,
            } => {
                assert_eq!(min_len, 5);
                assert_eq!(max_len, 10);
                match element.as_ref() {
                    Formula::Symbol(sym) => assert_eq!(sym.name(), "u32"),
                    _ => panic!("Expected Symbol"),
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_list_of_references() {
        let formula = parse_formula("[&Data; 1:100]").unwrap();
        match formula {
            Formula::List {
                element,
                min_len,
                max_len,
            } => {
                assert_eq!(min_len, 1);
                assert_eq!(max_len, 100);
                match element.as_ref() {
                    Formula::Reference { formula: inner } => match inner.as_ref() {
                        Formula::Symbol(sym) => assert_eq!(sym.name(), "Data"),
                        _ => panic!("Expected Symbol"),
                    },
                    _ => panic!("Expected Reference"),
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_deeply_nested() {
        let formula = parse_formula("[[u32]]").unwrap();
        match formula {
            Formula::List { element, .. } => match element.as_ref() {
                Formula::List { element: inner, .. } => match inner.as_ref() {
                    Formula::Symbol(sym) => assert_eq!(sym.name(), "u32"),
                    _ => panic!("Expected Symbol"),
                },
                _ => panic!("Expected List"),
            },
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_tuple_empty() {
        let formula = parse_formula("()").unwrap();
        match formula {
            Formula::Tuple { elements } => {
                assert_eq!(elements.len(), 0);
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_tuple_single_element() {
        let formula = parse_formula("(u32)").unwrap();
        match formula {
            Formula::Tuple { elements } => {
                assert_eq!(elements.len(), 1);
                match &elements[0] {
                    Formula::Symbol(sym) => assert_eq!(sym.name(), "u32"),
                    _ => panic!("Expected Symbol"),
                }
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_tuple_multiple_elements() {
        let formula = parse_formula("(u32, string, bool)").unwrap();
        match formula {
            Formula::Tuple { elements } => {
                assert_eq!(elements.len(), 3);
                match &elements[0] {
                    Formula::Symbol(sym) => assert_eq!(sym.name(), "u32"),
                    _ => panic!("Expected Symbol"),
                }
                match &elements[1] {
                    Formula::Symbol(sym) => assert_eq!(sym.name(), "string"),
                    _ => panic!("Expected Symbol"),
                }
                match &elements[2] {
                    Formula::Symbol(sym) => assert_eq!(sym.name(), "bool"),
                    _ => panic!("Expected Symbol"),
                }
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_tuple_with_complex_types() {
        let formula = parse_formula("([u32; 5], &Data)").unwrap();
        match formula {
            Formula::Tuple { elements } => {
                assert_eq!(elements.len(), 2);
                match &elements[0] {
                    Formula::List {
                        min_len, max_len, ..
                    } => {
                        assert_eq!(*min_len, 5);
                        assert_eq!(*max_len, 5);
                    }
                    _ => panic!("Expected List"),
                }
                match &elements[1] {
                    Formula::Reference { .. } => {}
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
            Formula::Record { fields } => {
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].name.as_str(), "x");
                match &fields[0].formula {
                    Formula::Symbol(sym) => assert_eq!(sym.name(), "u32"),
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
            Formula::Record { fields } => {
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
            Formula::Record { fields } => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name.as_str(), "data");
                match &fields[0].formula {
                    Formula::List {
                        min_len, max_len, ..
                    } => {
                        assert_eq!(*min_len, 10);
                        assert_eq!(*max_len, 10);
                    }
                    _ => panic!("Expected List"),
                }
                assert_eq!(fields[1].name.as_str(), "ptr");
                match &fields[1].formula {
                    Formula::Reference { .. } => {}
                    _ => panic!("Expected Reference"),
                }
            }
            _ => panic!("Expected Record"),
        }
    }

    #[test]
    fn test_nested_structures() {
        let formula = parse_formula("({x: u32, y: string})").unwrap();
        match formula {
            Formula::Tuple { elements } => {
                assert_eq!(elements.len(), 1);
                match &elements[0] {
                    Formula::Record { fields } => assert_eq!(fields.len(), 2),
                    _ => panic!("Expected Record"),
                }
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_variant_single() {
        let formula = parse_formula("|Foo u32").unwrap();
        match formula {
            Formula::Variant { variants } => {
                assert_eq!(variants.len(), 1);
                assert_eq!(variants[0].name.as_str(), "Foo");
                match &variants[0].formula {
                    Formula::Symbol(sym) => assert_eq!(sym.name(), "u32"),
                    _ => panic!("Expected Symbol"),
                }
            }
            _ => panic!("Expected Variant"),
        }
    }

    #[test]
    fn test_variant_multiple() {
        let formula = parse_formula("|None |Some u32").unwrap();
        match formula {
            Formula::Variant { variants } => {
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].name.as_str(), "None");
                assert_eq!(variants[1].name.as_str(), "Some");
            }
            _ => panic!("Expected Variant"),
        }
    }

    #[test]
    fn test_variant_with_complex_types() {
        let formula = parse_formula("|Ok {value: u32} |Err &string").unwrap();
        match formula {
            Formula::Variant { variants } => {
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].name.as_str(), "Ok");
                match &variants[0].formula {
                    Formula::Record { .. } => {}
                    _ => panic!("Expected Record"),
                }
                assert_eq!(variants[1].name.as_str(), "Err");
                match &variants[1].formula {
                    Formula::Reference { .. } => {}
                    _ => panic!("Expected Reference"),
                }
            }
            _ => panic!("Expected Variant"),
        }
    }

    #[test]
    fn test_module_simple() {
        let module = parse_module("formula MyType = u32;").unwrap();
        assert_eq!(module.formulas.len(), 1);
        assert_eq!(module.formulas[0].name.as_str(), "MyType");
        match &module.formulas[0].formula {
            Formula::Symbol(sym) => assert_eq!(sym.name(), "u32"),
            _ => panic!("Expected Symbol"),
        }
    }

    #[test]
    fn test_module_multiple_formulas() {
        let source = "formula A = u32; formula B = string; formula C = {x: u32};";
        let module = parse_module(source).unwrap();
        assert_eq!(module.formulas.len(), 3);
        assert_eq!(module.formulas[0].name.as_str(), "A");
        assert_eq!(module.formulas[1].name.as_str(), "B");
        assert_eq!(module.formulas[2].name.as_str(), "C");
    }
}
