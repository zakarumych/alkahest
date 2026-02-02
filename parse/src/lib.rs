//! Alkahest formulas parsing module.
//!
//!
//! This module provides functionality for parsing Alkahest formulas from authored source files.

#![no_std]

extern crate alloc;

// mod adt;
mod adt;
mod buffer;
mod lex;
mod parse;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[inline]
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    #[inline]
    pub fn join(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

pub use adt::{
    Definition, Element, ElementKind, Formula, ImportTree, List, Module, NamedElement,
    NamedVariant, ParseError, Path, Record, Tuple, Variant, Variants, parse_module,
};
