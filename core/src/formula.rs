use std::ops::{Add, AddAssign, Mul};

use crate::{DeserializeError, Deserializer};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizeBound {
    Unbounded,
    Bounded(usize),
    Exact(usize),
}

impl SizeBound {
    pub const fn add(self, rhs: SizeBound) -> SizeBound {
        match (self, rhs) {
            (SizeBound::Bounded(s), SizeBound::Bounded(r)) => SizeBound::Bounded(s + r),
            (SizeBound::Bounded(s), SizeBound::Exact(r)) => SizeBound::Bounded(s + r),
            (SizeBound::Exact(s), SizeBound::Bounded(r)) => SizeBound::Bounded(s + r),
            (SizeBound::Exact(s), SizeBound::Exact(r)) => SizeBound::Exact(s + r),
            _ => SizeBound::Unbounded,
        }
    }

    /// Returns the maximum of two size bounds.
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

    pub const fn mul(self, rhs: usize) -> SizeBound {
        match self {
            SizeBound::Unbounded => SizeBound::Unbounded,
            SizeBound::Bounded(size) => SizeBound::Bounded(size * rhs),
            SizeBound::Exact(size) => SizeBound::Exact(size * rhs),
        }
    }
}

impl Add<SizeBound> for SizeBound {
    type Output = SizeBound;

    #[inline]
    fn add(self, rhs: SizeBound) -> SizeBound {
        self.add(rhs)
    }
}

impl AddAssign<SizeBound> for SizeBound {
    #[inline]
    fn add_assign(&mut self, rhs: SizeBound) {
        *self = self.add(rhs);
    }
}

impl Mul<usize> for SizeBound {
    type Output = SizeBound;

    #[inline]
    fn mul(self, rhs: usize) -> SizeBound {
        match self {
            SizeBound::Bounded(size) => SizeBound::Bounded(size * rhs),
            SizeBound::Exact(size) => SizeBound::Exact(size * rhs),
            SizeBound::Unbounded => SizeBound::Unbounded,
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

pub struct SizeBytes<const SIZE_BYTES: u8>;

impl<const SIZE_BYTES: u8> SizeType for SizeBytes<SIZE_BYTES> {
    const VALUE: SizeBound = SizeBound::Exact(SIZE_BYTES as usize);
}

pub trait Formula: 'static {
    /// Stack size required for serializing this type.
    type StackSize<const SIZE_BYTES: u8>: SizeType + ?Sized;

    /// Heap size required for serializing this type.
    type HeapSize<const SIZE_BYTES: u8>: SizeType + ?Sized;

    /// Whether this formula is inhabited (i.e., has at least one valid value).
    /// Defaulted to true for convenience.
    const INHABITED: bool;
}
