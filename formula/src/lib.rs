use std::ops::Deref;

use crate::reference::Reference;

mod reference;

/// A reference to a formula.
#[derive(Clone)]
pub struct FormulaRef(Reference<Formula>);

impl FormulaRef {
    pub fn new(formula: Formula) -> Self {
        FormulaRef(Reference::new(formula))
    }

    pub const fn from_static(formula: &'static Formula) -> Self {
        FormulaRef(Reference::from_static(formula))
    }
}

impl Deref for FormulaRef {
    type Target = Formula;

    #[inline(always)]
    fn deref(&self) -> &Formula {
        self.0.as_ref()
    }
}

impl AsRef<Formula> for FormulaRef {
    #[inline(always)]
    fn as_ref(&self) -> &Formula {
        self.0.as_ref()
    }
}

/// This enum allows combining owned and static slices of formulas.
/// Static slices are used when formulas are built at compile time.
/// Owned slices are used when formulas are built at runtime.
#[derive(Clone)]
pub struct FormulaList(Reference<[Formula]>);

impl FormulaList {
    pub fn new(formulas: Vec<Formula>) -> Self {
        FormulaList(Reference::from_vec(formulas))
    }

    pub const fn from_static(formulas: &'static [Formula]) -> Self {
        FormulaList(Reference::from_static(formulas))
    }
}

impl Deref for FormulaList {
    type Target = [Formula];

    #[inline(always)]
    fn deref(&self) -> &[Formula] {
        self.0.as_ref()
    }
}

impl AsRef<[Formula]> for FormulaList {
    #[inline(always)]
    fn as_ref(&self) -> &[Formula] {
        self.0.as_ref()
    }
}

#[derive(Clone)]
pub struct Name(Reference<str>);

impl Name {
    pub fn new(name: &str) -> Self {
        Name(Reference::clone_from_str(name))
    }

    pub const fn from_static(name: &'static str) -> Self {
        Name(Reference::from_static(name))
    }
}

impl Deref for Name {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &str {
        self.0.as_ref()
    }
}

impl AsRef<str> for Name {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Clone)]
pub struct NamedFormula {
    pub name: Name,
    pub formula: Formula,
}

#[derive(Clone)]
pub struct NamedFormulaList(Reference<[NamedFormula]>);

impl NamedFormulaList {
    pub fn new(named_formulas: Vec<NamedFormula>) -> Self {
        NamedFormulaList(Reference::from_vec(named_formulas))
    }

    pub const fn from_static(named_formulas: &'static [NamedFormula]) -> Self {
        NamedFormulaList(Reference::from_static(named_formulas))
    }
}

impl Deref for NamedFormulaList {
    type Target = [NamedFormula];

    #[inline(always)]
    fn deref(&self) -> &[NamedFormula] {
        self.0.as_ref()
    }
}

impl AsRef<[NamedFormula]> for NamedFormulaList {
    #[inline(always)]
    fn as_ref(&self) -> &[NamedFormula] {
        self.0.as_ref()
    }
}

/// Primitive formula types.
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum PrimitiveFormula {
    /// Fixed-size integer formula.
    Integer {
        /// Signals that integer is signed.
        signed: bool,

        /// Number of bits in integer.
        bits: u32,
    },

    /// Fixed-size fixed-point number formula.
    FixedPoint {
        /// Signals that fixed-point number is signed.
        signed: bool,

        /// Number of bits in integer part.
        int_bits: u32,

        /// Number of bits in fractional part.
        frac_bits: u32,
    },

    /// Fixed-size floating-point number formula.
    Float {
        /// Number of bits in float.
        bits: u32,
    },

    /// Fixed-size rational number formula.
    Rational {
        /// Signals that numerator is signed.
        signed: bool,

        /// Number of bits in numerator.
        num_bits: u32,

        /// Number of bits in denominator.
        denom_bits: u32,
    },

    /// Variable-length number formula.
    VariableLengthNumber {
        /// Signals that number is signed.
        signed: bool,
    },

    /// Opaque blob of bytes with length bounds.
    ///
    /// Length bounds are inclusive.
    /// If `max_len` is `None` then there is no upper bound.
    ///
    /// Bounds affect how length is serialized.
    /// Unbounded length is serialized as VariableLengthNumber.
    Blob { min_len: u32, max_len: Option<u32> },

    /// UTF-8 encoded string with length bounds.
    /// To encode non-UTF-8 data use `Blob` formula instead.
    ///
    /// Length bounds are inclusive.
    /// If `max_len` is `None` then there is no upper bound.
    ///
    /// Bounds affect how length is serialized.
    /// Unbounded length is serialized as VariableLengthNumber.
    UTF8 {
        /// Minimum length of the string (inclusive).
        min_len: u32,

        /// Maximum length of the string (inclusive).
        /// If `None` then there is no upper bound.
        max_len: Option<u32>,
    },
}

/// Kind of the data formula.
#[derive(Clone)]
pub enum FormulaKind {
    /// Primitive formula not consisting of other formulas.
    Primitive { primitive: PrimitiveFormula },

    /// A formula of reference.
    /// Encoded as a pointer to another location that holds the actual data.
    Reference { formula: FormulaRef },

    /// A sequence of elements with the same formula.
    ///
    /// Length bounds are inclusive.
    ///
    /// Bounds affect how length is serialized.
    List {
        element: FormulaRef,
        min_len: u32,
        max_len: u32,
    },

    /// A fixed-length sequence of elements with different formulas.
    Tuple { elements: FormulaList },

    /// A fixed-length structure with named fields.
    Record { fields: NamedFormulaList },

    /// An alternative among different variants.
    Variant { variants: NamedFormulaList },
}

/// Describes a data formula.
/// Called "scheme" in some serialization libraries.
#[derive(Clone)]
pub struct Formula {
    pub name: Name,
    pub kind: FormulaKind,
}

impl Formula {
    /// Creates a new formula from its kind.
    pub const fn new(name: Name, kind: FormulaKind) -> Self {
        Formula { name, kind }
    }
}
