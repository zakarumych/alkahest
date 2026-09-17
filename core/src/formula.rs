#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizeBound {
    Unbounded,
    Bounded(usize),
    Exact(usize),
}

impl SizeBound {
    #[inline(always)]
    pub const fn is_unbounded(&self) -> bool {
        matches!(self, SizeBound::Unbounded)
    }

    #[inline(always)]
    pub const fn is_zero(&self) -> bool {
        matches!(self, SizeBound::Bounded(0) | SizeBound::Exact(0))
    }

    #[inline(always)]
    pub const fn add(self, rhs: SizeBound) -> SizeBound {
        match (self, rhs) {
            (SizeBound::Bounded(s), SizeBound::Bounded(r))
            | (SizeBound::Bounded(s), SizeBound::Exact(r))
            | (SizeBound::Exact(s), SizeBound::Bounded(r)) => match s.checked_add(r) {
                None => SizeBound::Unbounded,
                Some(sum) => SizeBound::Bounded(sum),
            },
            (SizeBound::Exact(s), SizeBound::Exact(r)) => match s.checked_add(r) {
                None => SizeBound::Unbounded,
                Some(sum) => SizeBound::Exact(sum),
            },
            _ => SizeBound::Unbounded,
        }
    }

    /// Returns the maximum of two size bounds.
    #[inline(always)]
    pub const fn max(self, rhs: SizeBound) -> SizeBound {
        match (self, rhs) {
            (SizeBound::Bounded(s), SizeBound::Bounded(r)) => {
                SizeBound::Bounded(if s > r { s } else { r })
            }
            (SizeBound::Bounded(s), SizeBound::Exact(r)) => {
                SizeBound::Bounded(if s > r { s } else { r })
            }
            (SizeBound::Exact(s), SizeBound::Bounded(r)) => {
                SizeBound::Bounded(if s > r { s } else { r })
            }
            (SizeBound::Exact(s), SizeBound::Exact(r)) => {
                if s == r {
                    SizeBound::Exact(s)
                } else {
                    SizeBound::Bounded(if s > r { s } else { r })
                }
            }
            _ => SizeBound::Unbounded,
        }
    }

    #[inline(always)]
    pub const fn mul(self, rhs: usize) -> SizeBound {
        match self {
            SizeBound::Unbounded => SizeBound::Unbounded,
            SizeBound::Bounded(size) => match size.checked_mul(rhs) {
                None => SizeBound::Unbounded,
                Some(product) => SizeBound::Bounded(product),
            },
            SizeBound::Exact(size) => match size.checked_mul(rhs) {
                None => SizeBound::Unbounded,
                Some(product) => SizeBound::Exact(product),
            },
        }
    }

    #[inline(always)]
    pub const fn not_exact(self) -> SizeBound {
        match self {
            SizeBound::Exact(size) => SizeBound::Bounded(size),
            other => other,
        }
    }

    #[inline(always)]
    pub const fn padded(self) -> SizeBound {
        match self {
            SizeBound::Bounded(size) => SizeBound::Exact(size),
            other => other,
        }
    }
}

/// Ad-hoc const arithmetics for size bounds.
pub trait SizeType {
    const VALUE: SizeBound;
}

pub struct UnboundedSize;

impl SizeType for UnboundedSize {
    const VALUE: SizeBound = SizeBound::Unbounded;
}

pub struct ExactSize<const SIZE: usize>;

impl<const SIZE: usize> SizeType for ExactSize<SIZE> {
    const VALUE: SizeBound = SizeBound::Exact(SIZE);
}

pub struct BoundedSize<const SIZE: usize>;

impl<const SIZE: usize> SizeType for BoundedSize<SIZE> {
    const VALUE: SizeBound = SizeBound::Bounded(SIZE);
}

pub trait Formula: AsFormula + 'static {
    /// Stack size required for serializing this type.
    type StackSize<const SIZE_BYTES: usize>: SizeType + ?Sized;

    /// Heap size required for serializing this type.
    type HeapSize<const SIZE_BYTES: usize>: SizeType + ?Sized;

    /// Whether this formula is inhabited (i.e., has at least one valid value).
    /// Defaulted to true for convenience.
    const INHABITED: bool;
}

/// Type that points at the formula type.
/// Formula types point to themselves.
pub trait AsFormula {
    type Formula: Formula + ?Sized;
}

impl<F> AsFormula for F
where
    F: Formula + ?Sized,
{
    type Formula = F;
}
