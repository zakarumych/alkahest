use core::{fmt, ops};

use crate::{
    buffer::{Buffer, BufferExhausted, CheckedFixedBuffer, DryBuffer, MaybeFixedBuffer},
    formula::Formula,
};

/// Heap and stack sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sizes {
    /// Heap size.
    pub heap: usize,

    /// Stack size.
    pub stack: usize,
}

impl Sizes {
    /// Zero sizes.
    pub const ZERO: Self = Sizes { heap: 0, stack: 0 };

    /// Create new `Sizes` with specified heap size.
    #[must_use]
    #[inline(always)]
    pub const fn with_heap(heap: usize) -> Self {
        Sizes { heap, stack: 0 }
    }

    /// Create new `Sizes` with specified stack size.
    #[must_use]
    #[inline(always)]
    pub const fn with_stack(stack: usize) -> Self {
        Sizes { heap: 0, stack }
    }

    /// Adds to the heap size.
    #[inline(always)]
    pub fn add_heap(&mut self, heap: usize) {
        self.heap += heap;
    }

    /// Adds to the stack size.
    #[inline(always)]
    pub fn add_stack(&mut self, stack: usize) {
        self.stack += stack;
    }

    /// Moves stack size to heap size.
    #[inline(always)]
    pub fn to_heap(&mut self, until: usize) -> usize {
        let len = self.stack - until;
        self.heap += len;
        self.stack = until;
        len
    }

    /// Returns total size.
    #[inline(always)]
    pub fn total(&self) -> usize {
        self.heap + self.stack
    }
}

impl ops::Add for Sizes {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self {
            heap: self.heap + rhs.heap,
            stack: self.stack + rhs.stack,
        }
    }
}

impl ops::AddAssign for Sizes {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.heap += rhs.heap;
        self.stack += rhs.stack;
    }
}

/// Trait for types that can be serialized with specified `F: `[`Formula`].
///
/// Implementations *must* ensure that serialized data conforms to formula layout.
///
/// # Examples
///
/// ```
/// # use alkahest::{*, advanced::*};
///
/// struct ThreeBytes;
///
/// impl Formula for ThreeBytes {
///     const MIN_STACK_SIZE: usize = 3;
///     const MAX_STACK_SIZE: Option<usize> = Some(3);
///     const HEAPLESS: bool = true;
/// }
///
/// struct Qwe;
///
/// impl Serialize<ThreeBytes> for Qwe {
///     fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
///     where
///         Self: Sized,
///         B: Buffer,
///     {
///         write_bytes(b"qwe", sizes, buffer)
///     }
///
///     fn size_hint(&self) -> Option<Sizes> {
///         Some(Sizes::with_stack(3))
///     }
/// }
/// ```
pub trait Serialize<F: ?Sized, const SIZE_BYTES: u8> {
    /// Serializes `self` into the given buffer.
    /// `heap` specifies the size of the buffer's heap occupied prior to this call.
    ///
    /// # Errors
    ///
    /// Returns error if buffer write fails.
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        Self: Sized,
        S: Serializer<SIZE_BYTES>;

    /// Returns heap and stack sizes required to serialize `self` according to formula `F`.
    ///
    /// Returns both stack and heap sizes required if any.
    ///
    /// This function may conservatively return `None` and it won't affect final serialization layout.
    /// However if sizes are known ahead of time, returning them may improve serialization performance.
    ///
    /// Returning incorrect sizes may lead to corrupted serialization or panics.
    fn size_hint(&self) -> Option<Sizes> {
        None
    }
}

/// Serialize value into buffer.
/// Returns total number of bytes written and size of the root value.
/// The buffer type controls bytes writing and failing strategy.
#[inline(always)]
pub fn serialize_into<F, T, B, const SIZE_BYTES: u8>(
    value: &T,
    buffer: B,
) -> Result<Sizes, B::Error>
where
    F: Formula<SIZE_BYTES> + ?Sized,
    T: Serialize<F, SIZE_BYTES>,
    B: Buffer,
{
    let mut sizes = Sizes { heap: 0, stack: 0 };

    let mut serializer = SerialzierImpl {
        sizes: &mut sizes,
        buffer,
    };

    serializer.write_indirect(value)?;
    Ok(sizes)
}

/// Serialize value into bytes slice.
/// Returns the number of bytes written.
/// Fails if the buffer is too small.
///
/// To retrieve the number of bytes required to serialize the value,
/// use [`serialized_size`] or [`serialize_or_size`].
///
/// # Errors
///
/// Returns [`BufferExhausted`] if the buffer is too small.
#[inline(always)]
pub fn serialize<F, T, const SIZE_BYTES: u8>(
    value: &T,
    output: &mut [u8],
) -> Result<Sizes, BufferExhausted>
where
    F: Formula<SIZE_BYTES> + ?Sized,
    T: Serialize<F, SIZE_BYTES>,
{
    serialize_into::<F, T, _, SIZE_BYTES>(value, CheckedFixedBuffer::new(output))
}

/// Slightly faster version of [`serialize`].
/// Panics if the buffer is too small instead of returning an error.
///
/// Use instead of using [`serialize`] with immediate [`unwrap`](Result::unwrap).
#[inline(always)]
pub fn serialize_unchecked<F, T, const SIZE_BYTES: u8>(value: &T, output: &mut [u8]) -> Sizes
where
    F: Formula<SIZE_BYTES> + ?Sized,
    T: Serialize<F, SIZE_BYTES>,
{
    match serialize_into::<F, T, _, SIZE_BYTES>(value, output) {
        Ok(sizes) => sizes,
        Err(never) => match never {},
    }
}

/// Error that may occur during serialization
/// if buffer is too small to fit serialized data.
///
/// Contains the size of the buffer required to fit serialized data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct BufferSizeRequired {
    /// Size of the buffer required to fit serialized data.
    pub required: usize,
}

impl fmt::Display for BufferSizeRequired {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "buffer size required: {}", self.required)
    }
}

/// Serialize value into bytes slice.
/// Returns the number of bytes written.
///
/// If the buffer is too small, returns error that contains
/// the exact number of bytes required.
///
/// Use [`serialize`] if this information is not needed.
///
/// # Errors
///
/// Returns [`BufferSizeRequired`] error if the buffer is too small.
/// Error contains the exact number of bytes required.
#[inline]
pub fn serialize_or_size<F, T, const SIZE_BYTES: u8>(
    value: &T,
    output: &mut [u8],
) -> Result<Sizes, BufferSizeRequired>
where
    F: Formula<SIZE_BYTES> + ?Sized,
    T: Serialize<F, SIZE_BYTES>,
{
    let mut exhausted = false;
    let result =
        serialize_into::<F, T, _, SIZE_BYTES>(value, MaybeFixedBuffer::new(output, &mut exhausted));
    let sizes = match result {
        Ok(sizes) => sizes,
        Err(never) => match never {},
    };
    if exhausted {
        Err(BufferSizeRequired {
            required: sizes.total(),
        })
    } else {
        Ok(sizes)
    }
}

/// Serialize value into byte vector.
/// Returns the number of bytes written.
///
/// Grows the vector if needed.
/// Infallible except for allocation errors.
///
/// Use pre-allocated vector when possible to avoid reallocations.
#[cfg(feature = "alloc")]
#[inline(always)]
pub fn serialize_to_vec<F, T, const SIZE_BYTES: u8>(
    value: &T,
    output: &mut alloc::vec::Vec<u8>,
) -> Sizes
where
    F: Formula<SIZE_BYTES> + ?Sized,
    T: Serialize<F, SIZE_BYTES>,
{
    use crate::buffer::VecBuffer;

    match serialize_into::<F, T, _, SIZE_BYTES>(value, VecBuffer::new(output)) {
        Ok(sizes) => sizes,
        Err(never) => match never {},
    }
}

/// Returns the number of bytes required to serialize the value.
/// Note that value is consumed.
///
/// Use when value is `Copy` or can be cheaply replicated to allocate
/// the buffer for serialization in advance.
/// Or to find out required size after [`serialize`] fails.
#[inline(always)]
pub fn serialized_sizes<F, T, const SIZE_BYTES: u8>(value: &T) -> Sizes
where
    F: Formula<SIZE_BYTES> + ?Sized,
    T: Serialize<F, SIZE_BYTES>,
{
    match serialize_into::<F, T, _, SIZE_BYTES>(value, DryBuffer) {
        Ok(sizes) => sizes,
        Err(never) => match never {},
    }
}

/// Size hint for serializing a field.
///
/// Use in [`Serialize::size_hint`](Serialize::size_hint) implementation.
#[inline]
pub fn field_size_hint<F: Formula<SIZE_BYTES> + ?Sized, const SIZE_BYTES: u8>(
    value: &impl Serialize<F, SIZE_BYTES>,
    last: bool,
) -> Option<Sizes> {
    match (last, F::MAX_STACK_SIZE) {
        (false, None) => None,
        (true, _) => {
            let sizes = value.size_hint()?;
            Some(sizes)
        }
        (false, Some(max_stack)) => {
            let sizes = value.size_hint()?;
            debug_assert!(sizes.stack <= max_stack);
            Some(Sizes {
                heap: sizes.heap,
                stack: max_stack,
            })
        }
    }
}

pub trait Serializer<const SIZE_BYTES: u8> {
    type Error;

    /// Serialize raw bytes.
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;

    /// Serialize element.
    ///
    /// This function is used when serializing fields of records, tuples, or elements of slices.
    fn write_direct<F, T>(&mut self, value: &T, last: bool) -> Result<(), Self::Error>
    where
        F: Formula<SIZE_BYTES> + ?Sized,
        T: Serialize<F, SIZE_BYTES>;

    /// Serialize element.
    ///
    /// This function is used when serializing fields of records, tuples, or elements of slices.
    fn write_indirect<F, T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        F: Formula<SIZE_BYTES> + ?Sized,
        T: Serialize<F, SIZE_BYTES>;

    /// Specialized method to write usize value in `SIZE_BYTES` bytes.
    fn write_usize(&mut self, value: usize) -> Result<(), Self::Error> {
        let max_size: usize = 1 << (SIZE_BYTES * 8);
        assert!(
            value < max_size,
            "Value too large to fit in SIZE_BYTES bytes ({SIZE_BYTES})"
        );

        let bytes = value.to_le_bytes();

        self.write_bytes(&bytes[..usize::from(SIZE_BYTES)])
    }
}

struct SerialzierImpl<'a, B: Buffer> {
    sizes: &'a mut Sizes,
    buffer: B,
}

impl<'a, B> SerialzierImpl<'a, B>
where
    B: Buffer,
{
    #[inline(always)]
    fn reborrow<'b>(&'b mut self) -> SerialzierImpl<'b, B::Reborrow<'b>>
    where
        B: Buffer,
    {
        SerialzierImpl {
            sizes: &mut *self.sizes,
            buffer: self.buffer.reborrow(),
        }
    }

    #[cold]
    #[inline(always)]
    fn write_to_heap<F, T, const SIZE_BYTES: u8>(&mut self, value: &T) -> Result<(), B::Error>
    where
        F: Formula<SIZE_BYTES> + ?Sized,
        T: Serialize<F, SIZE_BYTES>,
        B: Buffer,
    {
        let old_stack = self.sizes.stack;
        self.write_direct::<F, T>(value, true)?;
        let len = self.sizes.to_heap(old_stack);
        self.buffer
            .move_to_heap(self.sizes.heap - len, self.sizes.stack + len, len);
        Ok(())
    }
}

impl<'a, B, const SIZE_BYTES: u8> Serializer<SIZE_BYTES> for SerialzierImpl<'a, B>
where
    B: Buffer,
{
    type Error = B::Error;

    /// Write raw bytes to the buffer.
    ///
    /// Use in [`Serialize::serialize`](Serialize::serialize) implementation.
    ///
    /// # Errors
    ///
    /// Returns error if buffer write fails.
    #[inline(always)]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.buffer
            .write_stack(self.sizes.heap, self.sizes.stack, bytes)?;
        self.sizes.stack += bytes.len();
        Ok(())
    }

    /// Writes field value into the buffer.
    ///
    /// Use in [`Serialize::serialize`](Serialize::serialize) implementation.
    ///
    /// # Errors
    ///
    /// Returns error if buffer write fails.
    #[inline]
    fn write_direct<F, T>(&mut self, value: &T, last: bool) -> Result<(), Self::Error>
    where
        F: Formula<SIZE_BYTES> + ?Sized,
        T: Serialize<F, SIZE_BYTES>,
    {
        let old_stack = self.sizes.stack;

        <T as Serialize<F, SIZE_BYTES>>::serialize(
            value,
            SerialzierImpl {
                sizes: &mut *self.sizes,
                buffer: self.buffer.reborrow(),
            },
        )?;

        match (F::MAX_STACK_SIZE, F::EXACT_SIZE) {
            (None, _) => {}
            (Some(max_stack), false) => {
                debug_assert!(self.sizes.stack - old_stack <= max_stack);

                if !last {
                    self.buffer.pad_stack(
                        self.sizes.heap,
                        self.sizes.stack,
                        old_stack + max_stack - self.sizes.stack,
                    )?;
                    self.sizes.stack = old_stack + max_stack;
                }
            }
            (Some(exact_stack), true) => {
                debug_assert_eq!(self.sizes.stack - old_stack, exact_stack);
            }
        }

        Ok(())
    }

    /// Write value to the buffer as a reference,
    /// placing value into the heap and reference into the stack.
    ///
    /// Use in [`Serialize::serialize`](Serialize::serialize) implementation.
    ///
    /// # Errors
    ///
    /// Returns error if buffer write fails.
    #[inline]
    fn write_indirect<F, T>(&mut self, value: &T) -> Result<(), B::Error>
    where
        F: Formula<SIZE_BYTES> + ?Sized,
        T: Serialize<F, SIZE_BYTES>,
        B: Buffer,
    {
        // Can we get promised sizes?
        let promised = <T as Serialize<F, SIZE_BYTES>>::size_hint(&value);
        match promised {
            None => self.write_to_heap::<F, T, SIZE_BYTES>(value)?,
            Some(promised) => {
                match self.buffer.reserve_heap(
                    self.sizes.heap,
                    self.sizes.stack,
                    promised.total(),
                )? {
                    [] => {
                        let mut dry_serializer = SerialzierImpl {
                            sizes: &mut *self.sizes,
                            buffer: DryBuffer,
                        };
                        match dry_serializer.write_to_heap::<F, T, SIZE_BYTES>(value) {
                            Ok(stack) => stack,
                            Err(never) => match never {},
                        }
                    }
                    reserved => {
                        let mut reserved_sizes = Sizes {
                            heap: self.sizes.heap,
                            stack: 0,
                        };
                        <T as Serialize<F, SIZE_BYTES>>::serialize(
                            value,
                            SerialzierImpl {
                                sizes: &mut reserved_sizes,
                                buffer: reserved,
                            },
                        )
                        .expect("Reserved enough space");

                        debug_assert_eq!(reserved_sizes.heap, self.sizes.heap + promised.heap);
                        debug_assert_eq!(reserved_sizes.stack, promised.stack);

                        self.sizes.heap = reserved_sizes.total();
                    }
                }
            }
        };

        let address = self.sizes.heap;
        Serializer::<SIZE_BYTES>::write_usize(self, address)?;

        Ok(())
    }
}
